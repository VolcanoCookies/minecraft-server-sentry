pub mod client;
pub mod model;
pub mod packet;
pub mod response;
pub mod types;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures::future::join_all;
use kdam::term::Colorizer;
use kdam::{tqdm, BarExt, Column, RichProgress};
use model::player::{HistoricPlayer, OnlinePlayer};
use mongodb::bson::{doc, to_bson, DateTime};
use mongodb::options::UpdateOptions;
use mongodb::Client;
use mongodb::Collection;
use response::ResponseData;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio::{join, time};

use crate::model::server::Online;
use crate::packet::{handshake_status_packet, status_request_packet};
use crate::{
    model::{player::MinecraftPlayer, server::MinecraftServer},
    response::Response,
};

#[tokio::main]
async fn main() {
    let mut pb = RichProgress::new(
        tqdm!(
            total = 231231231,
            unit_scale = true,
            unit_divisor = 1024,
            unit = "B"
        ),
        vec![
            Column::Spinner(
                "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"
                    .chars()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>(),
                80.0,
                1.0,
            ),
            Column::text("[bold blue]?"),
            Column::Bar,
            Column::Percentage(1),
            Column::text("•"),
            Column::CountTotal,
            Column::text("•"),
            Column::RemainingTime,
        ],
    );

    pb.write("Connecting to mongodb".colorize("bold red"));

    let (servers, players) = connect_database(
        "mongodb://root:antek2015@localhost:27017/admin",
        "minecraft-server-entry",
    )
    .await;

    pb.write("Connected to mongodb".colorize("bold green"));

    let file = File::open("./masscan-out.txt").unwrap();
    let reader = BufReader::new(file).lines();

    pb.write("Collecting ips".colorize("bold red"));

    let mut hosts = Vec::<(String, i16)>::new();
    for line in reader {
        let l = line.unwrap();
        let mut spl = l.split(":");
        let ip = spl.next().unwrap();
        let port = spl.next().unwrap().parse::<i16>().unwrap();
        hosts.push((ip.to_owned(), port));
    }

    let total = hosts.len() as i32;

    pb.write(format!("Collected {} ips", total).colorize("bold green"));

    let (tx, mut rx) = mpsc::channel(1024);

    let handles: Vec<tokio::task::JoinHandle<()>> = hosts
        .iter()
        .map(move |(ip, port)| {
            let ip = ip.clone();
            let port = port.clone();
            let servers = servers.clone();
            let players = players.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                tx.send(0u8).await;

                let res = match connect(&ip, port).await {
                    Ok(res) => res,
                    Err(_) => {
                        tx.send(1u8).await;
                        //println!("{}:{} refused connection!", ip, port);
                        return;
                    }
                };

                handle_response(servers, players, res.data).await;
                tx.send(2u8).await;
            })
        })
        .collect();

    let join_task = tokio::spawn(async move {
        futures::future::join_all(handles).await;
    });

    pb.write("Scanning servers".colorize("bold blue"));

    let mut started = 0;
    let mut progress = 0;
    let mut last = Instant::now();
    while progress < total {
        let b = rx.recv().await.unwrap();
        match b {
            0u8 => started += 1,
            1u8 => progress += 1,
            2u8 => progress += 1,
            _ => {}
        }
        if last.elapsed().as_millis() > 200 {
            last = Instant::now();
            pb.update_to(progress as usize);
        }
    }

    join_task.await;

    pb.write("Finished scanning servers".colorize("bold green"));
}

async fn connect_database(
    uri: &str,
    database_name: &str,
) -> (Collection<MinecraftServer>, Collection<MinecraftPlayer>) {
    let mongo = Client::with_uri_str(uri).await.unwrap();
    let database = mongo.database(database_name);

    let servers = database.collection::<MinecraftServer>("servers");
    let players = database.collection::<MinecraftPlayer>("players");

    (servers, players)
}

async fn connect(ip: &str, port: i16) -> std::io::Result<Response> {
    //println!("Connecting to {}:{}", ip, port);

    let mut hostname = ip.to_owned();
    hostname.push_str(":");
    hostname.push_str(&port.to_string());
    let mut stream = TcpStream::connect(hostname).await?;

    let handshake_packet = handshake_status_packet(ip, port);
    let status_request_packet = status_request_packet();

    stream.write(&handshake_packet.to_bytes()).await?;
    stream.write(&status_request_packet.to_bytes()).await?;
    stream.flush().await?;

    let mut res = Response::read(&mut stream).await?;
    res.data.host = ip.to_owned();
    res.data.port = port;

    Ok(res)
}

async fn handle_response(
    servers: Collection<MinecraftServer>,
    players: Collection<MinecraftPlayer>,
    data: ResponseData,
) -> std::io::Result<()> {
    let online_players = data
        .players
        .list
        .iter()
        .cloned()
        .map(|p| Into::<OnlinePlayer>::into(p));

    let online = Online {
        max: data.players.max,
        players: data.players.online,
        list: HashSet::from_iter(online_players),
    };

    let mut set_on_insert = doc! {};

    set_on_insert.insert("host", data.host.clone());
    set_on_insert.insert("port", data.port as i32);
    set_on_insert.insert("whitelist", false);

    let mut set = doc! {};

    set.insert("online", to_bson(&online).unwrap());
    set.insert("motd", data.description.text());
    set.insert("version", to_bson(&data.version).unwrap());
    set.insert("last_updated", DateTime::now());
    set.insert("forge", data.forge_data.is_some());

    for online_player in &online.list {
        let key = format!("historic_players.{}", online_player.uuid.0);
        let historic_player = HistoricPlayer {
            uuid: online_player.uuid.clone(),
            last_seen: DateTime::now(),
        };
        set.insert(key, to_bson(&historic_player).unwrap());
    }

    let server_query = doc! {"host": data.host.clone(), "port": data.port as i32};
    let server_update = doc! {"$setOnInsert": set_on_insert, "$set": set};

    let server_future = servers.update_one(
        server_query,
        server_update,
        UpdateOptions::builder().upsert(true).build(),
    );
    let mut player_futures = Vec::new();

    for player in &data.players.list {
        let mut set_on_insert = doc! {};

        set_on_insert.insert("uuid", player.id.0.clone());

        let mut set = doc! {};

        set.insert("name", player.name.clone());
        set.insert("last_seen", DateTime::now());
        set.insert("last_updated", DateTime::now());

        let player_query = doc! {"uuid": player.id.0.clone()};
        let player_update = doc! {"$setOnInsert": set_on_insert, "$set": set};

        player_futures.push(players.update_one(
            player_query,
            player_update,
            UpdateOptions::builder().upsert(true).build(),
        ));
    }
    let (sres, pres) = join!(server_future, join_all(player_futures));

    if let Err(err) = sres {
        eprintln!("Error saving server to database: {}", err)
    }

    for res in pres {
        if let Err(err) = res {
            eprintln!("Error saving player to database: {}", err);
        }
    }

    Ok(())
}
