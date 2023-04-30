use core::time;
use redis::RedisError;
use serenity::{model::prelude::{ChannelId, GuildId, Mention, UserId}, prelude::Context};
use tracing::{info, error, warn};

use crate::{
    constants::ROULETTE_EXPIRE_TIME_SECONDS,
    utils::roulette::{get_spin, SpinResult, bet_check}, sql::{select::get_all_bets, structs::BetResult, delete::drop_old_bets},
};

use super::{get_conn, users::apply_winnings};

//check to see if there is a table
pub async fn table_exists(id: ChannelId) -> Result<bool, RedisError> {
    let mut conn = get_conn().await?;

    match redis::cmd("EXISTS")
        .arg(format!("roulette_{}", id.0))
        .query::<u8>(&mut conn)
    {
        Ok(e) => Ok(if e == 1 { true } else { false }),
        Err(e) => Err(e),
    }
}

//activate table mutex
pub async fn activate_table(id: ChannelId) -> Result<(), RedisError> {
    let mut conn = get_conn().await?;
    match redis::cmd("SET")
        .arg(format!("roulette_{}", id.0))
        .arg("1")
        .arg("EX")
        .arg(ROULETTE_EXPIRE_TIME_SECONDS)
        .query::<()>(&mut conn)
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

//initiate table_spin
pub async fn spin_table(guild_id: GuildId, id: ChannelId, ctx: Context) -> Result<Option<SpinResult>, RedisError> {
    if !match table_exists(id).await {
        Ok(e) => {
            info!("table active: {}", e);
            e
        }
        Err(e) => return Err(e),
    } {
        match activate_table(id).await {
            Ok(_) => {}
            Err(e) => return Err(e),
        };
        //start a new thread to spin the table
        let guild_id = guild_id.clone();
        let id = id.clone();
        tokio::spawn(async move {
            match spin_table_thread(guild_id, id, ctx).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Error spinning table: {}", e);
                }
            }
        });
        return Ok(None);
    }

    Ok(None)
}

async fn spin_table_thread(guild_id: GuildId, id: ChannelId, ctx: Context) -> Result<(), String> {
    info!("Spinning table for guild {} and channel {}", guild_id.0, id.0);
    //sleep for ROULETTE_EXPIRE_TIME_SECONDS
    tokio::time::sleep(time::Duration::from_secs(ROULETTE_EXPIRE_TIME_SECONDS)).await;
    //get spin
    let spin_result = get_spin();

    let bets = get_all_bets(id.0)
        .await
        .map_err(|e| e.to_string())?;

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

    //create embed    
    let mut embed = serenity::builder::CreateEmbed::default();
    embed.title("Roulette Results")
        .description(format!(
            "The results are in! The spin was {}!{}",
            spin_result,
            winners
        ))
        .color(serenity::utils::Colour::from_rgb(255, 0, 0));

    
    //apply the net to users and clear bet

    drop_old_bets(id)
        .await
        .map_err(|e| e.to_string())?;

    //apply nets
    apply_winnings(new_vec, guild_id).await;

    //send message containing embed

    match id
        .send_message(&ctx.http, |m| m.set_embed(embed))
        .await
    {
        Ok(_) => {}
        Err(e) => {
            warn!("Error sending message: {}", e);
        }
    }

    Ok(())
}
