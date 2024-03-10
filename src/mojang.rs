use std::io::Error;

use serde::Deserialize;
use serde_json::json;
use tracing::{instrument, trace};

const AUTH_URL: String = "https://authserver.mojang.com".to_owned();
const SESSION_URL: String = "https://sessionserver.mojang.com/session/minecraft/join".to_owned();

#[derive(Deserialize, Debug)]
#[serde(rename = "availableProfiles")]
struct AuthenticateResponse {
    user: User,
    client_token: String,
    access_token: String,
    available_profiles: Vec<Profile>,
    selected_profile: Profile,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "availableProfiles")]
struct User {
    username: String,
    id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "availableProfiles")]
struct Profile {
    name: String,
    id: String,
}

#[instrument]
pub async fn authenticate(
    username: &str,
    password: &str,
) -> Result<AuthenticateResponse, reqwest::Error> {
    trace!("Authenticating user: {}", username);

    let reqwest = reqwest::Client::new();

    let body = json!({
        "agent": {
            "name": "Minecraft",
            "version": 1
        },
        "username": username,
        "password": password,
        "requestUser": true
    });

    let response = reqwest
        .post(AUTH_URL + "/authenticate")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body).unwrap())
        .send()
        .await?;

    Ok(response.json().await?)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RefreshResponse {
    access_token: String,
    client_token: String,
    selected_profile: Profile,
    user: Option<User>,
    available_profiles: Vec<Profile>,
}

#[instrument]
pub async fn refresh(
    accessToken: &str,
    clientToken: &str,
) -> Result<AuthenticateResponse, reqwest::Error> {
    trace!("Refreshing access token");

    let reqwest = reqwest::Client::new();

    let body = json!({
        "accessToken": accessToken,
        "clientToken": clientToken,
        "requestUser": true
    });

    let response = reqwest
        .post(AUTH_URL + "/refresh")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body).unwrap())
        .send()
        .await?;

    Ok(response.json().await?)
}

#[instrument]
pub async fn validate(access_token: &str, client_token: &str) -> Result<(), Error> {
    trace!("Validating access token");

    let reqwest = reqwest::Client::new();

    let body = json!({
        "accessToken": access_token,
        "clientToken": client_token,
    });

    let response = reqwest
        .post(AUTH_URL + "/validate")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body).unwrap())
        .send()
        .await;

    if let Err(e) = response {
        return Err(Error::new(std::io::ErrorKind::Other, e.to_string()));
    }

    if response.unwrap().status() != 204 {
        Err(Error::new(
            std::io::ErrorKind::Other,
            "Invalid access token",
        ))
    } else {
        Ok(())
    }
}

#[instrument]
pub async fn join_server(
    access_token: &str,
    selected_profile: &str,
    server_hash: &str,
) -> Result<(), Error> {
    trace!("Joining server");

    let reqwest = reqwest::Client::new();

    let body = json!({
        "accessToken": access_token,
        "selectedProfile": selected_profile,
        "serverId": server_hash,
    });

    let response = reqwest
        .post(SESSION_URL)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body).unwrap())
        .send()
        .await;

    if let Err(e) = response {
        return Err(Error::new(std::io::ErrorKind::Other, e.to_string()));
    }

    if response.unwrap().status() != 204 {
        Err(Error::new(
            std::io::ErrorKind::Other,
            "Invalid access token",
        ))
    } else {
        Ok(())
    }
}
