//cleanup when first launch

use std::collections::HashMap;

use redis::RedisError;
use serenity::model::prelude::{GuildId, UserId};
use tracing::{info, warn};

use crate::{
    errors::GenericError,
    redis::cleanup::{remove_all_decks, remove_all_poker_hands},
    sql::{delete::delete_all_roulette_bets, select::get_all_open_roulette_bets},
    utils::money::change_balance,
};

pub async fn cleanup() -> Result<(), GenericError> {
    // Refund stale game state before wiping Redis so that we can read the
    // poker states and then delete the remaining keys.
    refund_stale_poker_states().await?;
    refund_open_roulette_bets().await?;

    match clean_redis().await {
        Ok(_) => Ok(()),
        Err(e) => Err(GenericError::new(&e.to_string())),
    }?;

    Ok(())
}

async fn clean_redis() -> Result<(), RedisError> {
    info!("Cleaning Redis");

    remove_all_decks().await?;
    remove_all_poker_hands().await?;

    info!("Done cleaning redis!");

    Ok(())
}

async fn refund_open_roulette_bets() -> Result<(), GenericError> {
    let bets = get_all_open_roulette_bets()
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    if bets.is_empty() {
        return Ok(());
    }

    info!("Refunding {} open roulette bets", bets.len());

    let mut refunds: HashMap<(u64, u64), i64> = HashMap::new();
    for bet in &bets {
        *refunds
            .entry((bet.user_id, bet.guild_id))
            .or_insert(0) += bet.net;
    }

    for ((user_id, guild_id), amount) in refunds {
        if let Err(e) = change_balance(
            UserId::from(user_id),
            GuildId::from(guild_id),
            amount,
            "refund",
            "roulette",
        )
        .await
        {
            warn!("Failed to refund roulette bet for {}: {}", user_id, e);
        }
    }

    delete_all_roulette_bets()
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    info!("Open roulette bets refunded");
    Ok(())
}

async fn refund_stale_poker_states() -> Result<(), GenericError> {
    let mut conn = crate::redis::get_conn()
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    let keys: Vec<String> = redis::cmd("KEYS")
        .arg("poker_state_*")
        .query(&mut conn)
        .map_err(|e| GenericError::new(&e.to_string()))?;

    if keys.is_empty() {
        return Ok(());
    }

    info!("Refunding {} stale poker states", keys.len());

    for key in keys {
        let raw: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query(&mut conn)
            .map_err(|e| GenericError::new(&e.to_string()))?;

        let Some(raw) = raw else { continue };
        let Ok(state) = serde_json::from_str::<crate::commands::poker::session::PokerGameState>(&raw) else {
            continue;
        };

        let gid = state_guild_id(&key);
        let guild_id = GuildId::from(gid);

        for (uid, amount) in &state.hand_bets {
            if *uid == crate::commands::poker::session::BOT_USER_ID {
                continue;
            }
            if let Err(e) = change_balance(
                UserId::from(*uid),
                guild_id,
                *amount as i64,
                "refund",
                "poker",
            )
            .await
            {
                warn!("Failed to refund poker bet for {}: {}", uid, e);
            }
        }

        // Delete all keys for this poker session
        let parts: Vec<&str> = key.split('_').collect();
        if parts.len() >= 4 {
            let gid = parts[2];
            let cid = parts[3];
            let _ = redis::cmd("DEL")
                .arg(format!("poker_state_{}_{}", gid, cid))
                .query::<()>(&mut conn);
            let _ = redis::cmd("KEYS")
                .arg(format!("poker_{}_{}_*", gid, cid))
                .query::<Vec<String>>(&mut conn)
                .map(|session_keys| {
                    let _: Result<(), _> = redis::cmd("DEL").arg(session_keys).query(&mut conn);
                });
        }
    }

    info!("Stale poker states refunded");
    Ok(())
}

fn state_guild_id(key: &str) -> u64 {
    key.split('_')
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

