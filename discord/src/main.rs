use std::{env, process::exit};

use models::Config;
use once_cell::sync::OnceCell;
use quote::process_message;
use serenity::{
    async_trait, framework::StandardFramework, futures::StreamExt, model::prelude::*, prelude::*,
    Client,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
// use warp::Filter;
mod models;
mod quote;
mod util;

struct Handler;

static POOL: OnceCell<Pool<Postgres>> = OnceCell::new();
static CONFIG: OnceCell<Config> = OnceCell::new();

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        println!(
            "new message '{}' by {} in {}",
            msg.content.replace("\n", " "),
            msg.author.name,
            msg.channel_id
        );
        let result = process_message(&ctx, msg, false).await;
        if let Err(why) = result {
            eprintln!("error processing message: {:?}", why);
        }
    }

    async fn ready(&self, ctx: Context, _: Ready) {
        let config = CONFIG.get().unwrap();

        let mut n_quotes: i64 = 0;
        let mut n_new_quotes: i64 = 0;
        let mut total: i64 = 0;
        for channel_id in &config.discord.target_channels {
            let channel_res = ChannelId(*channel_id).to_channel(&ctx).await;
            let maybe_channelname = channel_res
                .and_then(|c| {
                    c.guild()
                        .ok_or(serenity::Error::Other("This isn't a GuildChannel"))
                })
                .map(|gc| gc.name);

            println!(
                "checking channel {} ({:?})",
                channel_id,
                &maybe_channelname.unwrap_or_default()
            );

            let mut messages = ChannelId(*channel_id).messages_iter(&ctx).boxed();

            // TODO: make this all less ugly code + more robust (?)
            while let Some(message_result) = messages.next().await {
                total += 1;
                match message_result {
                    Ok(message) => {
                        let processing_res = process_message(&ctx, message, true).await;
                        if let Err(why) = processing_res {
                            eprintln!("error processing msg: {:?}", why);
                            continue;
                        }

                        let quote_info = processing_res.unwrap();
                        n_quotes += if quote_info.is_quote { 1 } else { 0 };
                        n_new_quotes += if !quote_info.is_duplicate { 1 } else { 0 };
                    }
                    Err(error) => eprintln!("error getting messages: {}", error),
                }
            }
        }

        let n_channels = &config.discord.target_channels.len();
        println!("{n_new_quotes} new quotes ({n_quotes} total) found in {total} messages of {n_channels} channels");
    }
}

#[tokio::main]
async fn main() {
    // read configuration
    let maybe_config = util::get_config();
    if let None = maybe_config {
        eprintln!("couldn't read config");
        exit(1);
    }
    CONFIG.set(maybe_config.unwrap()).unwrap();

    let config = CONFIG.get().unwrap();

    let pool = connect_db().await;
    POOL.set(pool).unwrap();

    // remove duplicates
    util::remove_duplicates().await.unwrap();

    // set up discord bot
    let framework = StandardFramework::new();

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES;
    let mut client = Client::builder(&config.discord.token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

async fn connect_db() -> Pool<Postgres> {
    dotenv::dotenv().ok();

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap()
}
