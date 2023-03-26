use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;

use tracing::{trace, Level};
use tracing_subscriber::{filter, fmt, prelude::*};

pub mod commands;
pub mod secrets;

use commands::help::*;

#[group]
#[commands(ping, help)]
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
    start_tracing();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    //get the login token from file
    let key = secrets::get_secret("disc_api").await;

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(key.value, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}
