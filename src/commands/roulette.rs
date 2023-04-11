use serenity::framework::standard::{Args, CommandResult};
//roulette wheel
use serenity::framework::standard::macros::command;
use serenity::model::prelude::{ChannelId, Mention, UserId};
use serenity::{model::prelude::Message, prelude::Context};
use tracing::info;
use tracing::log::warn;

use crate::redis::roulette::spin_table;
use crate::redis::users::apply_winnings;
use crate::sql::delete::drop_old_bets;
use crate::sql::insert::insert_roulette_bet;
use crate::sql::select::get_all_bets;
use crate::sql::structs::{BetResult, BettingTypes, RouletteBet};
use crate::utils::roulette::bet_check;

const USAGE_GENERAL: &str =
    "the command is roulette <command> <args>\n the available commands are as follows:
        - bet <bet> <amount>: place a <bet> on a color (red or black)
            or number (0-36) with <amount> greater than 10
        - odds <space>: get the odds of landing on <space>
        - table: view the bets currently on the table
        - timer: how much time is left till betting is closed";

const USAGE_ODDS: &str = "roulette odds <bet>";
const USAGE_BET: &str =
    " The command \"roulette\" must be in the specific format: (roulette <bet> <amount>)
        <bet>: must be a color (black or red) ora number (0-36)
        <amount>: the amount to bet, must be greater than 10";

const INVALID_BET: &str = "Invalid value entered for <bet> \n";
const INVALID_AMOUNT: &str = "Invalid value entered for <amount> \n";
const INVALID_COMMAND: &str = "Invalid value for <command>\n";

#[command]
pub async fn roulette(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let commands = vec![
        String::from("odds"),
        String::from("bet"),
        String::from("table"),
        String::from("timer"),
    ];
    //read the command
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

    let user_id = msg.author.id;
    let channel_id = msg.channel_id;

    //process the command
    match command.to_lowercase().as_str() {
        "bet" => make_bet(ctx, msg, args, &user_id).await,
        "odds" => odds(ctx, msg, args, &user_id).await,
        "table" => table(ctx, msg, &channel_id, &user_id).await,
        &_ => {
            msg.reply(ctx, "something went wrong").await?;
            return Ok(());
        }
    }
}

async fn make_bet(ctx: &Context, msg: &Message, mut args: Args, user_id: &UserId) -> CommandResult {
    //get the bet
    let bet = match args.single::<String>() {
        Ok(v) => v,
        Err(_) => {
            info!(
                "registered incorrect command \"{}\" in channel_id {}",
                msg.content, msg.channel_id
            );
            msg.reply(ctx, format!("{}{}", INVALID_BET, USAGE_BET))
                .await?;
            return Ok(());
        }
    };

    //get the amount
    let amount = match args.single::<i64>() {
        Ok(v) => v,
        Err(_) => {
            info!(
                "registered incorrect command \"{}\" in channel_id {}",
                msg.content, msg.channel_id
            );
            msg.reply(ctx, format!("{}{}", INVALID_AMOUNT, USAGE_BET))
                .await?;
            return Ok(());
        }
    };

    if amount <= 0 {
        msg.reply(ctx, "bet must be 1 or above!").await?;
        return Ok(());
    }

    //determine bet type

    let bet_type = match bet.to_uppercase().as_str() {
        "RED" => BettingTypes::RED,
        "BLACK" => BettingTypes::BLACK,
        "EVEN" => BettingTypes::EVEN,
        "ODD" => BettingTypes::ODD,
        b => match b.parse() {
            Ok(v) => {
                if 0 > v || v > 36 {
                    warn!(
                        "invalid value entered for bet by userid: {} in channel: {}",
                        user_id, msg.channel_id
                    );
                    msg.reply(ctx, format!("{}{}", INVALID_BET, USAGE_BET))
                        .await?;
                    return Ok(());
                }
                BettingTypes::SPECIFIC
            }
            Err(_) => {
                warn!(
                    "invalid value entered for bet by userid: {} in channel: {}",
                    user_id, msg.channel_id
                );
                msg.reply(ctx, format!("{}{}", INVALID_BET, USAGE_BET))
                    .await?;
                return Ok(());
            }
        },
    };

    let specific_bet: Option<u8> = match bet_type {
        BettingTypes::SPECIFIC => Some(bet.parse().unwrap()),
        _ => None,
    };

    let bet: RouletteBet = RouletteBet {
        amount,
        user_id: user_id.to_owned(),
        channel_id: msg.channel_id,
        bet_type,
        specific_bet,
    };

    let res = match insert_roulette_bet(bet).await {
        Err(e) => {
            warn!("unable to place bet {}", e);
            return Ok(());
        }
        Ok(_) => msg.reply(ctx, "Successfully placed bet!"),
    };

    res.await?;

    let spin = spin_table(msg.channel_id).await;
    let val = match spin {
        Ok(o) => o,
        Err(_) => {
            drop(spin);
            return Ok(());
        }
    };

    let spin_result = match val {
        Some(v) => v,
        None => {
            return Ok(());
        }
    };

    let bets = get_all_bets(msg.channel_id.0).await?;

    let mut winners = String::new();
    let mut new_vec: Vec<BetResult> = Vec::new();

    for mut bet in bets {
        bet_check(&mut bet, spin_result);
        new_vec.push(bet.clone());
        winners = format!(
            "{}\n{}",
            winners,
            format!(
                "{} {} {}!",
                Mention::from(UserId::from(bet.user_id)),
                if bet.net < 0 { "lost" } else { "won" },
                bet.net.abs()
            )
        )
    }

    let spin_message = msg.channel_id.say(
        ctx,
        format!(
            "The results are in! the wheel spun a value of {} {}! the results are as follows:\n{}",
            spin_result.color.to_string(),
            spin_result.value,
            winners,
        ),
    );
    //apply the net to users and clear bets

    drop_old_bets(msg.channel_id).await?;

    //apply nets
    apply_winnings(new_vec);

    spin_message.await?;

    Ok(())
}

async fn odds(ctx: &Context, msg: &Message, mut args: Args, user_id: &UserId) -> CommandResult {
    //get the bet
    let bet = match args.single::<String>() {
        Ok(v) => v,
        Err(_) => {
            info!(
                "registered incorrect command \"{}\" in channel_id {}",
                msg.content, msg.channel_id
            );
            msg.reply(ctx, format!("{}{}", INVALID_BET, USAGE_ODDS))
                .await?;
            return Ok(());
        }
    };

    info!("request for odds on bet {} by userId {}", bet, user_id);
    Ok(())
}

async fn table(
    ctx: &Context,
    msg: &Message,
    channel_id: &ChannelId,
    user_id: &UserId,
) -> CommandResult {
    info!(
        "requesting information on table in channel {} by userId {}",
        channel_id, user_id
    );

    msg.reply(ctx, "PLACEHOLDER").await?;

    Ok(())
}
