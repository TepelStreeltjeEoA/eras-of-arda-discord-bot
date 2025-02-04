use serde::{Deserialize, Serialize};
use serenity::client::Context;

use crate::constants::MINECRAFT_API;
use crate::get_reqwest_client;

#[derive(Serialize, Deserialize, Debug)]
pub struct Description {
    pub raw: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerList {
    pub online: u32,
    pub max: u32,
    pub list: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftServer {
    pub online: bool,
    pub motd: Description,
    pub players: PlayerList,
}

pub async fn get_server_status(ctx: &Context, ip: &str) -> Option<MinecraftServer> {
    let rclient = get_reqwest_client!(ctx);

    let req = format!("{}{}", MINECRAFT_API, ip);
    let res = rclient.get(&req).send().await.ok()?.text().await.ok()?;
    if let Ok(server) = serde_json::from_str::<MinecraftServer>(&res) {
        if server.online {
            Some(server)
        } else {
            None
        }
    } else {
        None
    }
}
