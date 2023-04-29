use std::process::exit;

use serenity::async_trait; 


use serenity::model::prelude::Ready;
use serenity::model::prelude::interaction::{Interaction};
use serenity::prelude::*;

use tracing::log::{error};
use tracing::{info, Level};
use tracing_subscriber::{filter, fmt, prelude::*};

pub mod commands;
pub mod constants;
pub mod errors;
pub mod redis;
pub mod secrets;
pub mod sql;
pub mod utils;


use utils::cleanup::cleanup;
use utils::command_handler::command_handler;
struct Handler;

//event handler for the bot with slash commands

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            //pass it to the command handler
            match command_handler(command,&ctx).await{
                Ok(_) => {}
                Err(e) => {
                    error!("Error handling command: {}", e);
                }
            };
        }
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        //register the slash commands using register_commands
        if let Err(why) = commands::register_commands(&ctx).await {
            error!("Could not register commands: {}", why);
        }
    }
}

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

    match redis::test_connection().await {
        Ok(_) => info!("Successfully connected to redis"),
        Err(e) => {
            error!("Could not connect to redis database");
            error!("{}", e);
            exit(1);
        }
    }

    match cleanup().await {
        Ok(_) => {
            info!("Successfully cleaned up!")
        }
        Err(e) => {
            error!("Could not clean up");

            //if e contains the number 10054 then it is likely due to redis being in protected mode
            if e.to_string().contains("10054") {
                error!("Redis is in protected mode. Please disable it");
            }

            error!("{}", e);
        }
    }


    // TODO refund all open tables

    //get the login token from file
    let key = secrets::get_secret("disc_api");

    let mut client = Client::builder(key.value, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        error!("An error occurred while running the client: {:?}", why);
    }
}
