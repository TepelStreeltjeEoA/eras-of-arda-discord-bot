use bytesize::ByteSize;
use serenity::builder::CreateBotAuthParameters;
use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::interactions::message_component::ButtonStyle;
use serenity::model::prelude::*;
use serenity::utils::{colours, Colour};

use crate::api::curseforge;
use crate::check::*;
use crate::constants::{CURSEFORGE_ID_LEGACY, CURSEFORGE_ID_RENEWED};

#[command]
#[only_in(guilds)]
pub async fn renewed(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(Colour::DARK_GOLD);
                e.title("Renewed First Age and Second Age");
                e.description(
                    "The 1.16.5 addon is available on [CurseForge](https://www.curseforge.com/minecraft/mc-mods/the-first-age-submod) \
                    It currently contains the First Age \
                    It will contain the Second Age in the future. \
                    \nFor a list of features present in the Renewed addon, check the announcements channel.",
                )
            });

            m
        })
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn legacy(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(Colour::DARK_GOLD);
                e.title("Legacy First Age and Second Age");
                e.description(
                    "The Legacy versions of our submods can be found on CurseForge \
                    They will not be updated frequently and only receive bug fixes \
                    [Legacy First Age](https://www.curseforge.com/minecraft/mc-mods/the-first-age-submod) \
                    [Legacy Second Age](https://www.curseforge.com/minecraft/mc-mods/eras-of-arda-the-second-age-submod) \
                    \nAny future updates will be announced in the announcements channel.",
                )
            });

            m
        })
        .await?;

    Ok(())
}

fn pretty_large_int<T: Into<u128>>(x: T) -> String {
    let mut num = x.into();
    let mut s = String::new();
    while num / 1000 != 0 {
        s = format!(",{:03}{}", num % 1000, s);
        num /= 1000;
    }
    format!("{}{}", num % 1000, s)
}

#[command]
#[aliases("download")]
pub async fn curseforge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let id = if args.single::<String>().unwrap_or_default().to_lowercase() == "renewed" {
        CURSEFORGE_ID_RENEWED
    } else {
        CURSEFORGE_ID_LEGACY
    };
    let project = curseforge::get_project_info(ctx, id).await?;

    if project.data.id != id {
        println!("=== ERROR ===\nCurseforge API call returned the wrong project\n=== END ===");
        return Ok(());
    }

    let file = if let Some(file) = project.data.latest_files.get(0) {
        file
    } else {
        println!("=== ERROR ===\nNo Curseforge latest file\n=== END ===");
        return Ok(());
    };

    let url = format!(
        "{}/files/{}",
        project.data.links.website_url.trim_end_matches('/'),
        file.id
    );

    let mod_version = format!(
        "Download {} {}",
        if id == CURSEFORGE_ID_LEGACY {
            "Legacy"
        } else {
            "Renewed"
        },
        file.file_name
            .rfind(&[' ', '-', '_', '+', 'v'][..])
            .map(|i| file.file_name[i + 1..].trim_end_matches(".jar"))
            .unwrap_or_default()
    );

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.thumbnail(&project.data.logo.thumbnail_url);
                e.author(|a| {
                    a.name("Curseforge")
                        .icon_url(crate::constants::CURSEFORGE_ICON)
                })
                .colour(Colour(0xf16436))
                .title(&project.data.name)
                .url(&project.data.links.website_url)
                .description(&project.data.summary)
                .field(
                    "Latest Version",
                    format!(
                        "[{}]({}) ({})",
                        file.file_name,
                        url,
                        ByteSize(file.file_length)
                    ),
                    false,
                )
                .footer(|f| {
                    f.text(format!(
                        "Total download count: {}",
                        pretty_large_int(project.data.download_count as u64)
                    ))
                })
                .timestamp(file.file_date)
            })
            .components(|c| {
                c.create_action_row(|a| {
                    a.create_button(|b| b.style(ButtonStyle::Link).label(mod_version).url(&url))
                })
            })
        })
        .await
        .unwrap();

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn forge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let (version, mc) = if args.single::<String>().unwrap_or_default() == "legacy" {
        ("1614", "1.7.10")
    } else {
        ("36.2.0", "1.16.5")
    };
    let forge_link = crate::constants::FORGE_LINK.replace("{mc}", mc);
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(Colour::DARK_BLUE)
                    .title("Have you checked your Forge version?")
                    .description(format!(
                        "To function properly, the mod needs to run with \
Forge {} or later for Minecraft {}",
                        version, mc
                    ))
                    .author(|a| {
                        a.name(format!("Minecraft Forge for {}", mc))
                            .icon_url(crate::constants::FORGE_ICON)
                            .url(&forge_link)
                    })
            })
            .components(|c| {
                c.create_action_row(|b| {
                    b.create_button(|b| {
                        b.style(ButtonStyle::Link)
                            .label(format!("Download Forge for {}", mc))
                            .url(&forge_link)
                    })
                })
            })
        })
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn coremod(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(Colour::RED);
                e.title("Check your mod file extension!");
                e.description(
                    "Sometimes when downloading the mod with a browser like Firefox, the mod \
file is saved with a `.zip` extension instead of a `.jar`
When this happens, the mod will not function properly: among other things that will happen, mod \
fences and fence gates will not connect, and horses will go very slowly.

To fix this, go to your `/.minecraft/mods` folder and change the file extension!",
                )
            })
        })
        .await?;
    Ok(())
}

#[command]
#[checks(allowed_blacklist)]
pub async fn invite(ctx: &Context, msg: &Message) -> CommandResult {
    let user_icon = ctx.cache.current_user_field(|user| user.face());
    let invite_url = {
        let mut builder = CreateBotAuthParameters::default();
        builder
            .permissions(crate::constants::INVITE_PERMISSIONS)
            .auto_client_id(ctx)
            .await?
            .scopes(&[OAuth2Scope::Bot, OAuth2Scope::ApplicationsCommands]);
        builder.build()
    };

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(colours::branding::BLURPLE)
                    .author(|a| {
                        a.name("LOTR Mod Bot");
                        a.icon_url(user_icon)
                    })
                    .field(
                        "Invite me to your server!",
                        format!("My invite link is available [here]({}).", invite_url),
                        false,
                    )
            })
            .components(|c| {
                c.create_action_row(|a| {
                    a.create_button(|b| {
                        b.style(ButtonStyle::Link)
                            .label("Invite me")
                            .url(invite_url)
                    })
                })
            })
        })
        .await?;

    Ok(())
}

#[command]
pub async fn discord(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(
            ctx,
            "The invite for the **Eras of Arda Community Discord** is available here:
https://discord.gg/kQAh9eh",
        )
        .await?;
    Ok(())
}

#[command]
#[aliases("fb")]
pub async fn facebook(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(Colour::new(0x1877F2));
                e.description(
                    "Check the mod's Facebook page for
updates and teasers [here](https://www.facebook.com/erasofarda/)!",
                );
                e.thumbnail(crate::constants::FACEBOOK_ICON);
                e.title("Link to the Facebook page");
                e.url("https://www.facebook.com/erasofarda/");
                e
            })
            .components(|c| {
                c.create_action_row(|a| {
                    a.create_button(|b| {
                        b.style(ButtonStyle::Link)
                            .label("Facebook page")
                            .url("https://www.facebook.com/erasofarda/")
                    })
                })
            })
        })
        .await?;
    Ok(())
}

#[command]
#[aliases("donation", "paypal")]
pub async fn donate(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(Colour::new(0xCEBD9C));
                e.description(
                    "Donations will be thanked with the Supporter Shield \
[Shield](https://erasofarda.fandom.com/wiki/Donator_Shield).",
                );
                e.thumbnail(crate::constants::DONATE_THUMBNAIL);
                e.title("Donate to the mod!");
                e
            })
            .components(|c| {
                c.create_action_row(|a| {
                    use crate::constants::*;
                    a.create_button(|b| {
                        b.style(ButtonStyle::Link)
                            .label("Donate in $")
                            .url(PAYPAL_LINK_DOLLARS)
                    })
                    .create_button(|b| {
                        b.style(ButtonStyle::Link)
                            .label("Donate in £")
                            .url(PAYPAL_LINK_POUNDS)
                    })
                    .create_button(|b| {
                        b.style(ButtonStyle::Link)
                            .label("Donate in €")
                            .url(PAYPAL_LINK_EUROS)
                    })
                })
            })
        })
        .await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(allowed_blacklist)]
#[aliases("user")]
pub async fn user_info(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_id = if let Some(user) = msg.mentions.first() {
        user.id
    } else if let Ok(user_id) = args.single::<UserId>() {
        user_id
    } else {
        msg.author.id
    };
    let member = msg
        .guild_id
        .unwrap_or_default()
        .member(ctx, user_id)
        .await?;
    let user = &member.user;

    let colour = member.colour(ctx).unwrap_or_default();

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(colour);
                e.thumbnail(user.face());
                if let Some(nick) = &member.nick {
                    e.title(nick);
                    e.description(format!(
                        "Username: **{}**{}",
                        &user.name,
                        if user.bot {
                            "\n_This user is a bot_"
                        } else {
                            ""
                        }
                    ));
                } else {
                    e.title(&user.name);
                    if user.bot {
                        e.description("_This user is a bot_");
                    }
                }
                e.field(
                    "Account creation date",
                    &user.id.created_at().format("%d %B %Y at %R"),
                    true,
                );
                if let Some(joined_at) = member.joined_at {
                    e.field(
                        "Account join date",
                        joined_at.format("%d %B %Y at %R"),
                        true,
                    );
                }
                if !member.roles.is_empty() {
                    e.field(
                        "Roles",
                        member
                            .roles
                            .iter()
                            .map(|r| r.mention().to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                        false,
                    );
                }
                e
            })
        })
        .await?;
    Ok(())
}
