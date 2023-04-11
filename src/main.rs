use std::process::exit;

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;

use tracing::log::error;
use tracing::{info, Level};
use tracing_subscriber::{filter, fmt, prelude::*};

pub mod commands;
pub mod constants;
pub mod errors;
pub mod redis;
pub mod secrets;
pub mod sql;
pub mod utils;

use commands::help::*;
use commands::money::*;
use commands::roulette::*;

#[group]
#[commands(ping, help, roulette, bal)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

fn start_tracing() {
    //we just want trace on for my base crate and nothing else
    let filter = filter::Targets::default().with_target("gambling_bot", Level::TRACE);

    //create and register the logger
    tracing_subscriber::Registry::default()
        .with(fmt::layer())
        .with(filter)
        .init();
}

#[tokio::main]
async fn main() {
    //start logging
    start_tracing();

    //test the db connection
    match sql::test_connection().await {
        Ok(_) => info!("Successfully connected to database"),
        Err(e) => {
            error!("Could not connect to sql database");
            error!("{}", e);
            exit(1);
        }
    }

    match redis::test_connection() {
        Ok(_) => info!("Successfully connected to redis"),
        Err(e) => {
            error!("Could not connect to sql database");
            error!("{}", e);
            exit(1);
        }
    }

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    //get the login token from file
    let key = secrets::get_secret("disc_api");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(key.value, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        error!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}
