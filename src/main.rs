pub mod client;
pub mod model;
mod mojang;
pub mod packet;
pub mod packet_types;
pub mod response;

use std::collections::HashSet;
use std::fs::File;
use std::io::{stdout, BufRead, BufReader, Stdout};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use client::MinecraftClient;
use futures::future::join_all;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use indicatif::MultiProgress;
use kdam::term::Colorizer;
use kdam::{tqdm, BarExt, Column, RichProgress};
use model::player::{HistoricPlayer, OnlinePlayer};
use mongodb::bson::{doc, to_bson, DateTime};
use mongodb::options::UpdateOptions;
use mongodb::Client;
use mongodb::Collection;
use opentelemetry::global::GlobalTracerProvider;
use opentelemetry::trace::TracerProvider;
use opentelemetry_stdout::SpanExporter;
use response::ResponseData;
use simple_logger::SimpleLogger;
use time::macros::format_description;
use tokio::io::AsyncWriteExt;
use tokio::join;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Semaphore};
use tokio::time::Instant;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

use crate::model::server::Online;
use crate::packet::{handshake_status_packet, status_request_packet};
use crate::{
    model::{player::MinecraftPlayer, server::MinecraftServer},
    response::Response,
};

#[tokio::main]
async fn main() {
    let collector = tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::TRACE)
        // build but do not install the subscriber.
        .init();
    /*
    // Create a new OpenTelemetry trace pipeline that prints to stdout
    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_simple_exporter(SpanExporter::default())
        .build();
    let tracer = provider.tracer("readme_example");

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    let subscriber = Registry::default().with(telemetry);
    */

    let logger = SimpleLogger::new()
        .env()
        .with_level(log::LevelFilter::Info)
        .with_module_level("serenity", log::LevelFilter::Warn)
        .with_module_level("tracing", log::LevelFilter::Warn)
        .with_timestamp_format(format_description!(
            "[[[year]-[month]-[day] [hour]:[minute]:[second]]"
        ));
    let multi = MultiProgress::new();

    //LogWrapper::new(multi.clone(), logger).try_init().unwrap();

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

    let ip_addr = std::net::IpAddr::V4(Ipv4Addr::new(91, 134, 157, 209));
    let port = 25602;
    let addr = SocketAddr::new(ip_addr, port);
    let mut client = MinecraftClient::new();
    let res = client.connect(addr).await;

    if let Err(err) = res {
        eprintln!("Error connecting to server: {}", err);
        return;
    }

    unreachable!("This code is unreachable");

    pb.write("Connecting to mongodb".colorize("bold red"));

    let mongo_url = std::env::var("MONGO_URL")
        .unwrap_or("mongodb://root:antek2015@localhost:27017/admin".to_owned());
    let database_name =
        std::env::var("DATABASE_NAME").unwrap_or("minecraft-server-sentry".to_owned());

    let (servers, players) = connect_database(&mongo_url, &database_name).await;

    pb.write("Connected to mongodb".colorize("bold green"));

    pb.write("Collecting ips".colorize("bold red"));
    let file = File::open("./masscan-out.txt").unwrap();
    let reader = BufReader::new(file).lines();

    let mut hosts = Vec::<(String, i16)>::new();
    for line in reader {
        let l = line.unwrap();
        let mut spl = l.split(":");
        let ip = spl.next().unwrap();
        let port = spl.next().unwrap().parse::<i16>().unwrap();
        hosts.push((ip.to_owned(), port));
    }

    let total = hosts.len() as i32;
    pb.pb.set_total(total as usize);

    pb.write(format!("Collected {} ips", total).colorize("bold green"));

    let (tx, mut rx) = mpsc::channel(1024);

    let semaphore = Arc::new(Semaphore::new(10000));
    let mut tasks = FuturesUnordered::new();
    let handle = tokio::spawn(async move {
        for (ip, port) in hosts.into_iter() {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let servers = servers.clone();
            let players = players.clone();
            let tx = tx.clone();
            tasks.push(tokio::spawn(async move {
                tx.send(0u8).await;
                let res = match connect(&ip, port).await {
                    Ok(res) => {
                        println!("{}:{} accepted the connection!", ip, port);
                        res
                    }
                    Err(_) => {
                        tx.send(1u8).await;
                        println!("{}:{} refused connection!", ip, port);
                        return;
                    }
                };
                handle_response(servers, players, res.data).await;
                tx.send(2u8).await;
                drop(permit);
            }));
        }

        while let Some(_) = tasks.next().await {}
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
        if last.elapsed().as_millis() > 50 {
            last = Instant::now();
            pb.update_to(progress as usize);
        }
    }

    let _ = handle.await;

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

    let timeout = tokio::time::timeout(Duration::from_secs(10), TcpStream::connect(hostname)).await;

    let mut stream = match timeout {
        Ok(Ok(stream)) => stream,
        Ok(Err(err)) => return Err(err),
        Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout")),
    };

    let handshake_packet = handshake_status_packet(ip, port);
    let status_request_packet = status_request_packet();

    stream.write_all(&handshake_packet.to_bytes()).await?;
    stream.write_all(&status_request_packet.to_bytes()).await?;
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
