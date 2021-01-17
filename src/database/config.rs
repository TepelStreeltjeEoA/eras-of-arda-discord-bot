use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::id::GuildId;

use super::DatabasePool;
use crate::constants::{TABLE_MC_SERVER_IP, TABLE_PREFIX};

pub async fn get_prefix(ctx: &Context, guild_id: Option<GuildId>) -> Option<String> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>()?.clone()
    };
    let mut conn = pool.get_conn().await.ok()?;

    let server_id: u64 = guild_id?.0;

    let res = conn
        .query_first(format!(
            "SELECT prefix FROM {} WHERE server_id={}",
            TABLE_PREFIX, server_id
        ))
        .await;

    drop(conn);

    if let Ok(Some(prefix)) = res {
        prefix
    } else {
        println!("Initializing prefix for {:?}", guild_id);
        set_prefix(ctx, guild_id, "!", false).await.ok()?;
        println!("Prefix initialized successfully");
        Some("!".to_string())
    }
}

pub async fn set_prefix(
    ctx: &Context,
    guild_id: Option<GuildId>,
    prefix: &str,
    update: bool,
) -> CommandResult {
    let pool = {
        let data_read = ctx.data.read().await;
        if let Some(p) = data_read.get::<DatabasePool>() {
            p.clone()
        } else {
            println!("Could not retrieve the database pool");
            return Ok(());
        }
    };
    let mut conn = pool.get_conn().await?;

    let server_id: u64 = guild_id.unwrap_or(GuildId(0)).0;

    let req = if update {
        println!("Updating prefix to \"{}\"", prefix);
        format!(
            "UPDATE {} SET prefix = :prefix WHERE server_id = :server_id",
            TABLE_PREFIX
        )
    } else {
        println!("Initializing prefix to \"{}\"", prefix);
        format!(
            "INSERT INTO {} (server_id, prefix) VALUES (:server_id, :prefix)",
            TABLE_PREFIX
        )
    };
    conn.exec_drop(
        req.as_str(),
        params! {
            "server_id" => server_id,
            "prefix" => prefix,
        },
    )
    .await?;
    println!("Done.");

    drop(conn);

    Ok(())
}

pub async fn get_minecraft_ip(ctx: &Context, guild_id: Option<GuildId>) -> Option<String> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>()?.clone()
    };
    let mut conn = pool.get_conn().await.ok()?;

    let server_id: u64 = guild_id?.0;

    let ip = conn
        .query_first(format!(
            "SELECT mc_ip FROM {} WHERE server_id={}",
            TABLE_MC_SERVER_IP, server_id
        ))
        .await
        .ok()?;

    drop(conn);

    ip
}

pub async fn set_minecraft_ip(
    ctx: &Context,
    guild_id: Option<GuildId>,
    ip: &String,
    update: bool,
) -> CommandResult {
    let pool = {
        let data_read = ctx.data.read().await;
        if let Some(p) = data_read.get::<DatabasePool>() {
            p.clone()
        } else {
            println!("Could not retrieve the database pool");
            return Ok(());
        }
    };
    let mut conn = pool.get_conn().await?;

    let server_id: u64 = guild_id.map(|g| g.0).unwrap_or_default();

    let req = if update {
        println!("Updating IP to {}", ip);
        format!(
            "UPDATE {} SET mc_ip = :mc_ip WHERE server_id = :server_id",
            TABLE_MC_SERVER_IP
        )
    } else {
        println!("Setting up ip to {}", ip);
        format!(
            "INSERT INTO {} (server_id, mc_ip) VALUES (:server_id, :mc_ip)",
            TABLE_MC_SERVER_IP
        )
    };
    conn.exec_drop(
        req.as_str(),
        params! {
            "server_id" => server_id,
            "mc_ip" => ip,
        },
    )
    .await?;
    println!("Done");

    drop(conn);

    Ok(())
}

pub async fn delete_minecraft_ip(ctx: &Context, guild_id: Option<GuildId>) -> CommandResult {
    let pool = {
        let data_read = ctx.data.read().await;
        if let Some(p) = data_read.get::<DatabasePool>() {
            p.clone()
        } else {
            println!("Could not retrieve the database pool");
            return Ok(());
        }
    };
    let mut conn = pool.get_conn().await?;
    let server_id: u64 = guild_id.map(|g| g.0).unwrap_or_default();

    let req = format!(
        "DELETE FROM {} WHERE server_id = :server_id LIMIT 1",
        TABLE_MC_SERVER_IP
    );

    conn.exec_drop(
        req.as_str(),
        params! {
            "server_id" => server_id
        },
    )
    .await?;

    drop(conn);
    Ok(())
}
