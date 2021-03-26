use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;

use crate::api;
use api::structures::*;
use api::structures::{Lang::*, Namespace::*};

async fn wiki_search(
    ctx: &Context,
    msg: &Message,
    args: Args,
    namespace: Namespace,
    wiki: &Wikis,
) -> CommandResult {
    let srsearch = args.rest();
    println!("srsearch {}", srsearch);
    let p = api::search(ctx, &namespace, srsearch, wiki).await;
    if let Some(page) = p {
        api::display(ctx, msg, &page, wiki).await?;
    } else {
        msg.reply(
            ctx,
            format!("Couldn't find a {} for the given name!", namespace),
        )
        .await?;
    }
    Ok(())
}

fn lang(mut args: Args) -> (Lang, Args, bool) {
    let mut default = false;
    let lang = match args
        .single::<String>()
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "en" | "english" => En,
        "fr" | "french" => Fr,
        "es" | "spanish" => Es,
        "de" | "german" => De,
        "nl" | "dutch" => Nl,
        "zh" | "chinese" => Zh,
        "ru" | "russian" => Ru,
        "ja" | "japanese" => Ja,
        a => {
            println!("{}", a);
            default = true;
            En
        }
    };
    (lang, args, default)
}

async fn lotr_wiki(ctx: &Context, msg: &Message, args: Args, ns: Namespace) -> CommandResult {
    let res = lang(args);
    let lang = res.0;
    let mut args = res.1;
    let default = res.2;
    let wiki = Wikis::LOTRMod(lang);
    if default {
        println!("rewinding");
        args.rewind();
    }
    if !args.is_empty() {
        wiki_search(ctx, msg, args, ns, &wiki).await?;
    } else {
        api::display(ctx, msg, &ns.main_page(&wiki, &msg.author.name), &wiki).await?;
    }
    Ok(())
}

#[command]
#[sub_commands(discord_link)]
pub async fn wiki(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, args, Page).await?;
    Ok(())
}

#[command]
#[aliases("discord")]
pub async fn discord_link(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(
            ctx,
            "The invite for the **LOTR Mod Community Discord** is available here:
https://discord.gg/QXkZzKU",
        )
        .await?;
    Ok(())
}

#[command]
pub async fn user(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, args, User).await?;
    Ok(())
}

#[command]
pub async fn category(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, args, Category).await?;
    Ok(())
}
#[command]
pub async fn template(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, args, Template).await?;
    Ok(())
}

#[command]
pub async fn file(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, args, File).await?;
    Ok(())
}

#[command]
pub async fn random(ctx: &Context, msg: &Message) -> CommandResult {
    let wiki = &Wikis::LOTRMod(En);
    let p = api::random(ctx, wiki).await;
    if let Some(page) = p {
        api::display(ctx, msg, &page, wiki).await?;
    } else {
        msg.channel_id.say(ctx, "Couldn't execute query!").await?;
    }
    Ok(())
}

#[command]
pub async fn tolkien(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let wiki = Wikis::TolkienGateway;
    if !args.is_empty() {
        wiki_search(ctx, msg, args, Page, &wiki).await?;
    } else {
        api::display(ctx, msg, &wiki.default(&msg.author.name), &wiki).await?;
    }
    Ok(())
}

#[command]
pub async fn minecraft(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let wiki = Wikis::Minecraft;
    if !args.is_empty() {
        wiki_search(ctx, msg, args, Page, &wiki).await?;
    } else {
        api::display(ctx, msg, &wiki.default(&msg.author.name), &wiki).await?;
    }
    Ok(())
}
