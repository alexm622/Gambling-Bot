//poker

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::{ChannelId, Message, UserId},
    prelude::Context,
};
use tracing::{info, log::warn};

use crate::{
    errors::GenericError,
    redis::{
        decks::draw_card,
        poker::{get_user_hand, push_poker_hand},
    },
    sql::structs::{poker_hand_to_emojis, PokerHand},
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
        .say(
            ctx,
            format!("your hand is:\n{}", poker_hand_to_emojis(hand)),
        )
        .await?;

    Ok(())
}

//deal in player
pub async fn deal(uid: UserId, cid: ChannelId) -> Result<PokerHand, GenericError> {
    //temporary function to check deck creation
    let hand = match get_user_hand(cid, uid).await {
        Ok(v) => v,
        Err(e) => return Err(GenericError::new(&e.to_string())),
    };

    Ok(hand)
}

#[command]
pub async fn discard(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let num_args = args.len();

    if num_args == 0 {
        msg.channel_id
            .say(ctx, "please enter an integer for discard")
            .await?;
        return Ok(());
    }

    //get the user's hand

    let mut hand = get_user_hand(msg.channel_id, msg.author.id).await?;

    let mut discard: Vec<u8> = Vec::new();

    for _ in 0..num_args {
        let arg = match args.single::<u8>() {
            Ok(v) => v,
            Err(_) => {
                msg.channel_id
                    .say(ctx, "please enter an integer for discard")
                    .await?;
                return Ok(());
            }
        };
        discard.push(arg);
    }

    info!("len:{}", num_args);

    discard
        .clone()
        .into_iter()
        .for_each(|va| info!("value of {}", va));

    for i in discard {
        match i {
            1 => hand.one = draw_card(msg.channel_id, 0, 1).await?,
            2 => hand.two = draw_card(msg.channel_id, 0, 1).await?,
            3 => hand.three = draw_card(msg.channel_id, 0, 1).await?,
            4 => hand.four = draw_card(msg.channel_id, 0, 1).await?,
            5 => hand.five = draw_card(msg.channel_id, 0, 1).await?,
            _ => {
                msg.channel_id
                    .say(ctx, format!("{} is not a valid number", i))
                    .await?;
                return Ok(());
            }
        }
    }
    msg.channel_id
        .say(
            ctx,
            format!("your hand is now:\n{}", poker_hand_to_emojis(hand)),
        )
        .await?;

    push_poker_hand(hand, msg.channel_id, msg.author.id).await?;

    Ok(())
}

#[command]
pub async fn pinfo(ctx: &Context, msg: &Message) -> CommandResult {
    let hand = get_user_hand(msg.channel_id, msg.author.id).await?;

    msg.reply(
        ctx,
        format!("your hand is:\n{}", poker_hand_to_emojis(hand)),
    )
    .await?;

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
