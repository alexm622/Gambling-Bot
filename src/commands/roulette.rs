use serenity::framework::standard::{Args, CommandResult};
//roulette wheel
use serenity::framework::standard::macros::command;
use serenity::model::prelude::{ChannelId, UserId};
use serenity::{model::prelude::Message, prelude::Context};
use tracing::info;

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

    if !commands.contains(&command) {
        info!(
            "registered incorrect command \"{}\" in channel_id {}",
            msg.content, msg.channel_id
        );
        msg.reply(ctx, format!("{}{}", INVALID_COMMAND, USAGE_GENERAL))
            .await?;
        return Ok(());
    }

    let bet = match args.single::<String>() {
        Ok(v) => v,
        Err(_) => {
            info!(
                "registered incorrect command \"{}\" in channel_id {}",
                msg.content, msg.channel_id
            );
            msg.reply(ctx, format!("{}{}", INVALID_BET, USAGE_GENERAL))
                .await?;
            return Ok(());
        }
    };
    let amount = match args.single::<u32>() {
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

    info!(
        "registered command \"{}\" with bet on \"{}\" of amount {}",
        command, bet, amount
    );
    Ok(())
}

async fn make_bet(user_id: &UserId, bet: &str, amount: u32) {
    info!("placed bet of {},{} for userId {}", bet, amount, user_id);
}

async fn odds(ctx: &Context, msg: &Message, bet: &str, user_id: &UserId) -> CommandResult {
    info!("request for odds on bet {} by userId {}", bet, user_id);
    Ok(())
}

async fn table(ctx: &Context, msg: &Message, channel_id: ChannelId, user_id: &UserId) {
    info!(
        "requesting information on table in channel {} by userId {}",
        channel_id, user_id
    );
}
