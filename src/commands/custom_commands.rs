use serde_json::Value;
use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;

use crate::announcement::announce;
use crate::check::IS_ADMIN_CHECK;
use crate::constants::{MANAGE_BOT_PERMS, OWNER_ID, RESERVED_NAMES};
use crate::database::{
    blacklist::check_blacklist,
    custom_commands::{
        add_custom_command, check_command_exists, get_command_data, get_custom_commands_list,
        remove_custom_command,
    },
    Blacklist,
};
use crate::utils::{get_json_from_message, has_permission, JsonMessageError::*};
use crate::{failure, handle_json_error, is_admin, success};

#[command]
#[only_in(guilds)]
#[aliases("command")]
#[sub_commands(define, custom_command_remove, custom_command_display)]
pub async fn custom_command(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("Custom command execution...");
    let name = args.single::<String>()?.to_lowercase();
    if let Some(command_data) = get_command_data(ctx, msg.guild_id, &name, false).await {
        let mut body = if command_data.body.contains('$') {
            command_data
                .body
                .replace('$', "\u{200B}$")
                .replace("\\\u{200B}$", "\\$")
        } else {
            command_data.body
        };
        for (i, arg) in args.iter::<String>().enumerate() {
            body = body.replace(
                format!("\u{200B}${}", i).as_str(),
                &arg?
                    .as_str()
                    .trim_matches('"')
                    .replace('$', "\\$")
                    .replace('@', "@\u{200B}")
                    .replace('\\', "\\\\")
                    .replace('\n', "\\n")
                    .replace('"', "\\\""),
            );
        }
        println!("{}", body);
        let mut message: Value = serde_json::from_str(&body.replace("\\$", "$"))?;
        // early interrupt in case of blacklist / admin command
        if let Value::String(s) = &message["type"] {
            let is_admin = msg.author.id == OWNER_ID
                || is_admin!(ctx, msg)
                || has_permission(ctx, msg.guild_id, msg.author.id, *MANAGE_BOT_PERMS).await;
            if !is_admin {
                if s == "meme"
                    && check_blacklist(ctx, msg, false)
                        .await
                        .unwrap_or(Blacklist::IsBlacklisted(true))
                        .is_blacklisted()
                {
                    msg.delete(ctx).await?;
                    msg.author
                        .dm(ctx, |m| {
                            m.content("You are not allowed to use this command here!")
                        })
                        .await?;
                    return Ok(());
                } else if s == "admin" {
                    failure!(ctx, msg, "You are not an admin on this server!");
                }
            }
        }
        // default args
        if let Value::Array(a) = &message["default_args"] {
            let argc = args.len() - 1;
            let changed = body.contains(format!("\u{200B}${}", argc).as_str());
            for (i, arg) in a[argc.min(a.len())..].iter().enumerate() {
                if let Value::String(s) = arg {
                    println!("Default argument '{}'", s);
                    body = body.replace(
                        format!("\u{200B}${}", i + argc).as_str(),
                        &s.replace('$', "\\$"),
                    );
                }
            }
            if changed {
                message = serde_json::from_str(&body.replace("\\$", "$"))?;
            }
        }
        let delete = message["self_delete"].as_bool().unwrap_or_default();
        announce(ctx, msg.channel_id, message).await?;
        if delete {
            msg.delete(ctx).await?;
        }
    } else {
        println!("Could not find custom command \"{}\"", name);
    }
    Ok(())
}

#[command]
#[checks(is_admin)]
pub async fn define(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name: String = args.single::<String>()?.to_lowercase();
    if RESERVED_NAMES.contains(&name.as_str()) {
        failure!(
            ctx,
            msg,
            "You cannot add a command with the reserved name `{}`",
            name
        );
        return Ok(());
    }
    let update = check_command_exists(ctx, msg.guild_id, &name)
        .await
        .unwrap_or_default();

    match get_json_from_message(msg).await {
        Ok(mut message) => {
            let documentation = message
                .as_object_mut()
                .map(|map| map.remove("documentation").unwrap_or_default())
                .unwrap_or_default();
            let body = serde_json::to_string_pretty(&message)?;
            println!(
                "adding custom command \"{}\": {}\n({:?})",
                name, body, documentation
            );
            let db_res = add_custom_command(
                ctx,
                msg.guild_id,
                &name,
                &body,
                documentation.as_str(),
                update,
            )
            .await;
            if db_res.is_ok()
                && check_command_exists(ctx, msg.guild_id, &name)
                    .await
                    .unwrap_or_default()
            {
                success!(ctx, msg);
            } else {
                println!("{:?}", db_res.err());
                failure!(ctx, msg);
            }
        }
        Err(e) => handle_json_error!(ctx, msg, e),
    }
    Ok(())
}

#[command]
#[checks(is_admin)]
#[aliases("remove", "delete")]
async fn custom_command_remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name: String = args.single()?;
    if check_command_exists(ctx, msg.guild_id, &name)
        .await
        .unwrap_or_default()
        && remove_custom_command(ctx, msg.guild_id, &name)
            .await
            .is_ok()
        && !check_command_exists(ctx, msg.guild_id, &name)
            .await
            .unwrap_or_default()
    {
        success!(ctx, msg);
    } else {
        failure!(ctx, msg);
    }
    Ok(())
}

#[command]
#[aliases("display", "show")]
#[checks(is_admin)]
async fn custom_command_display(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(name) = args.single::<String>() {
        if let Some(command) = get_command_data(ctx, msg.guild_id, &name, true).await {
            println!("Displaying command docs...");
            msg.channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.title(format!("Custom command: {}", name));
                        if let Some(desc) = command.description {
                            e.description(desc);
                        }

                        e.field(
                            "Command body",
                            if command.body.len() < 1013 {
                                format!(
                                    "```json\n{}```",
                                    command.body.replace("```", "`\u{200B}``")
                                )
                            } else {
                                "_Too long to display here_".into()
                            },
                            false,
                        )
                    })
                })
                .await?;
        } else {
            failure!(ctx, msg, "The custom command does not exist!");
        }
    } else if let Some(list) = get_custom_commands_list(ctx, msg.guild_id).await {
        println!("displaying a list of custom commands");
        let mut newline = 0;
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title("Custom commands");
                    e.description(
                        list.iter()
                            .map(|(name, desc)| {
                                format!(
                                    "{skip}`{}`  {}\n",
                                    name,
                                    if desc.is_empty() {
                                        newline += 1;
                                        "_No description_"
                                    } else {
                                        desc
                                    },
                                    skip = if newline == 1 {
                                        newline += 1;
                                        "\n"
                                    } else {
                                        ""
                                    }
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(""),
                    )
                })
            })
            .await?;
    }
    Ok(())
}
