//poker

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::{Channel, ChannelId, Message, UserId},
    prelude::Context,
};
use tracing::{info, log::warn};

use crate::{
    errors::GenericError,
    redis::{decks::set_deck, poker::get_user_hand},
    sql::structs::PokerHand,
    utils::{generate_deck, SIZE_POKER},
};

const USAGE_GENERAL: &str =
    "the command is poker <command> <args>\n the available commands are as follows:
        - play
        - discard
        - check
        - raise";

const INVALID_BET: &str = "Invalid value entered for <bet> \n";
const INVALID_AMOUNT: &str = "Invalid value entered for <amount> \n";
const INVALID_COMMAND: &str = "Invalid value for <command>\n";

const CANT_RAISE: &str = "Insufficient funds to raise\n";
const CONFIRM_FOLD: &str = "Are you sure that you want to fold? (y/n)";

#[command]
pub async fn poker(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let commands = vec![
        String::from("play"),
        String::from("discard"),
        String::from("check"),
        String::from("raise"),
        String::from("fold"),
    ];

    //get the first command
    let command = match args.single::<String>() {
        Ok(v) => v,
        Err(_) => {
            info!(
                "registered incorrect command \"{}\" in channel_id {}",
                msg.content, msg.channel_id
            );
            msg.reply(ctx, format!("{}{}", INVALID_COMMAND, USAGE_GENERAL))
                .await?;
            return Ok(());
        }
    };

    //if its an unknown command then print message
    if !commands.contains(&command) {
        info!(
            "registered incorrect command \"{}\" in channel_id {}",
            msg.content, msg.channel_id
        );
        msg.reply(ctx, format!("{}{}", INVALID_COMMAND, USAGE_GENERAL))
            .await?;
        return Ok(());
    }

    match command.to_lowercase().as_str() {
        "play" => pplay(ctx, msg, args).await,
        "discard" => discard(ctx, msg, args).await,
        "check" => check(ctx, msg, args).await,
        "raise" => raise(ctx, msg, args).await,
        "fold" => fold(ctx, msg, args).await,
        &_ => {
            msg.reply(ctx, "something went wrong").await?;
            return Ok(());
        }
    }
}

#[command]
pub async fn pplay(ctx: &Context, msg: &Message) -> CommandResult {
    let msg_id = msg.id;

    //Message::reply(msg, ctx, "attempting to deal you in").await?;
    let hand = match deal(msg.author.id, msg.channel_id).await {
        Ok(v) => v,
        Err(e) => {
            warn!("encountered error: {}", e);
            return Ok(());
        }
    };
    info!("got hand {}", hand);
    msg.channel_id
        .say(ctx, format!("your hand is: {}", hand.to_string()))
        .await?;

    Ok(())
}

//deal in player
pub async fn deal(uid: UserId, cid: ChannelId) -> Result<PokerHand, GenericError> {
    //temporary function to check deck creation
    let hand = match get_user_hand(cid, uid) {
        Ok(v) => v,
        Err(e) => return Err(GenericError::new(&e.to_string())),
    };

    Ok(hand)
}

#[command]
pub async fn discard(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    Ok(())
}

#[command]
pub async fn check(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    Ok(())
}

#[command]
pub async fn raise(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    Ok(())
}

#[command]
pub async fn fold(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    Ok(())
}
