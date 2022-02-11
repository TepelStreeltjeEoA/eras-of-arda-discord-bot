use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;

use crate::{api, failure};
use api::wiki;
use api::wiki::structures::{Lang, Lang::*, Namespace, Namespace::*, Wikis};

async fn wiki_search(
    ctx: &Context,
    msg: &Message,
    args: &mut Args,
    namespace: Namespace,
    wiki: &Wikis,
) -> CommandResult {
    let srsearch = args.rest();
    let p = wiki::search(ctx, &namespace, srsearch, wiki).await;
    if let Some(page) = p {
        wiki::display(ctx, msg, &page, wiki).await?;
    } else {
        failure!(
            ctx,
            msg,
            "Couldn't find a {} for the given name!",
            namespace
        );
    }
    Ok(())
}

fn lang(args: &mut Args) -> Option<Lang> {
    Some(
        match args.single::<String>().ok()?.to_lowercase().as_str() {
            "en" | "english" => En,
            "fr" | "french" => Fr,
            "es" | "spanish" => Es,
            "de" | "german" => De,
            "nl" | "dutch" => Nl,
            "zh" | "chinese" => Zh,
            "ru" | "russian" => Ru,
            "ja" | "japanese" => Ja,
            _ => {
                args.rewind();
                return None;
            }
        },
    )
}

async fn lotr_wiki(ctx: &Context, msg: &Message, args: &mut Args, ns: Namespace) -> CommandResult {
    let language = lang(args).unwrap_or_default();
    let wiki = Wikis::LotrMod(language);
    if !args.is_empty() {
        wiki_search(ctx, msg, args, ns, &wiki).await
    } else {
        wiki::display(ctx, msg, &ns.main_page(&wiki, &msg.author.name), &wiki).await
    }
}

async fn eoa_wiki(ctx: &Context, msg: &Message, args: &mut Args, ns: Namespace) -> CommandResult {
    if !args.is_empty() {
        wiki_search(ctx, msg, args, ns, &Wikis::EoA).await
    } else {
        wiki::display(
            ctx,
            msg,
            &ns.main_page(&Wikis::EoA, &msg.author.name),
            &Wikis::EoA,
        )
        .await
    }
}

#[command]
#[sub_commands(
    eoa_user,
    eoa_category,
    eoa_template,
    eoa_file,
    eoa_random,
    lotrmod,
    tolkien,
    minecraft
)]
pub async fn wiki(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    eoa_wiki(ctx, msg, &mut args, Page).await
}

#[command("user")]
async fn eoa_user(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    eoa_wiki(ctx, msg, &mut args, User).await
}

#[command("category")]
async fn eoa_category(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    eoa_wiki(ctx, msg, &mut args, Category).await
}
#[command("template")]
async fn eoa_template(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    eoa_wiki(ctx, msg, &mut args, Template).await
}

#[command("file")]
async fn eoa_file(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    eoa_wiki(ctx, msg, &mut args, File).await
}

#[command("random")]
async fn eoa_random(ctx: &Context, msg: &Message) -> CommandResult {
    let wiki = &Wikis::EoA;
    let p = wiki::random(ctx, wiki).await;
    if let Some(page) = p {
        wiki::display(ctx, msg, &page, wiki).await?;
    } else {
        failure!(ctx, msg, "Couldn't execute query!");
    }
    Ok(())
}

#[command]
#[aliases("lotr")]
#[sub_commands(lotr_user, lotr_category, lotr_template, lotr_file, lotr_random)]
pub async fn lotrmod(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, &mut args, Page).await
}

#[command("user")]
async fn lotr_user(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, &mut args, User).await
}

#[command("category")]
async fn lotr_category(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, &mut args, Category).await
}
#[command("template")]
async fn lotr_template(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, &mut args, Template).await
}

#[command("file")]
async fn lotr_file(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, &mut args, File).await
}

#[command("random")]
async fn lotr_random(ctx: &Context, msg: &Message) -> CommandResult {
    let wiki = &Wikis::LotrMod(En);
    let p = wiki::random(ctx, wiki).await;
    if let Some(page) = p {
        wiki::display(ctx, msg, &page, wiki).await?;
    } else {
        failure!(ctx, msg, "Couldn't execute query!");
    }
    Ok(())
}

#[command]
#[aliases("tolkiengateway")]
pub async fn tolkien(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let wiki = Wikis::TolkienGateway;
    if !args.is_empty() {
        wiki_search(ctx, msg, &mut args, Page, &wiki).await?;
    } else {
        wiki::display(ctx, msg, &wiki.default(&msg.author.name), &wiki).await?;
    }
    Ok(())
}

#[command]
#[aliases("mc")]
pub async fn minecraft(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let wiki = Wikis::Minecraft;
    if !args.is_empty() {
        wiki_search(ctx, msg, &mut args, Page, &wiki).await?;
    } else {
        wiki::display(ctx, msg, &wiki.default(&msg.author.name), &wiki).await?;
    }
    Ok(())
}
