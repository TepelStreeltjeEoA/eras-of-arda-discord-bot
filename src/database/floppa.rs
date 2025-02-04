use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;

use crate::constants::{OWNER_ID, TABLE_ADMINS, TABLE_FLOPPA};
use crate::get_database_conn;

pub async fn get_floppa(ctx: &Context, n: Option<i64>) -> Option<String> {
    let mut conn = get_database_conn!(ctx);

    if let Some(n) = n {
        let max_len: i64 = conn
            .query_first(format!("SELECT MAX(id) FROM {}", TABLE_FLOPPA))
            .await
            .ok()??;
        let num = (((n - 1) % max_len) + max_len) % max_len + 1;
        conn.query_first(format!(
            "SELECT image_url FROM {} WHERE id={}",
            TABLE_FLOPPA, num
        ))
        .await
    } else {
        conn.query_first(format!(
            "SELECT image_url FROM {} ORDER BY RAND() LIMIT 1 ",
            TABLE_FLOPPA
        ))
        .await
    }
    .ok()?
}

pub async fn add_floppa(ctx: &Context, floppa_url: String) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    let images: Vec<String> = conn
        .exec_map(
            format!("SELECT image_url FROM {}", TABLE_FLOPPA).as_str(),
            (),
            |url| url,
        )
        .await?;

    println!("Retrieved floppa urls");

    if !images.contains(&floppa_url) {
        conn.exec_drop(
            format!(
                "INSERT INTO {} (image_url) VALUES (:image_url)",
                TABLE_FLOPPA
            )
            .as_str(),
            params! {
                "image_url" => floppa_url,
            },
        )
        .await?;
        println!("Successfully executed query!");
    } else {
        OWNER_ID
            .to_user(ctx)
            .await?
            .dm(ctx, |m| {
                m.content("Tried to add floppa that already exists!")
            })
            .await?;
    }

    drop(conn);

    Ok(())
}

pub async fn is_floppadmin(ctx: &Context, server_id: GuildId, user_id: UserId) -> Option<bool> {
    let mut conn = get_database_conn!(ctx);

    let res = conn
        .query_first(
            format!(
                "SELECT EXISTS(SELECT user_id FROM {} WHERE server_id={} AND user_id={} AND floppadmin = true LIMIT 1)",
                TABLE_ADMINS, server_id.0, user_id.0
            )
        )
        .await
        .ok()?;

    drop(conn);

    res
}
