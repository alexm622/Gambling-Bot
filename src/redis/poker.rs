//poker

use redis::RedisError;
use serenity::model::prelude::{ChannelId, GuildId};

use super::get_conn;

/// Delete all poker-related keys for a channel, including keys left behind by
/// older versions of the bot.
pub async fn cleanup_game(gid: GuildId, cid: ChannelId) -> Result<(), RedisError> {
    let mut conn = get_conn().await?;
    let patterns = vec![
        format!("poker_{}_{}_*", gid, cid),
        format!("poker_joinned_{}_{}", gid, cid),
        format!("poker_joinable_{}_{}", gid, cid),
        format!("poker_current_bet_{}_{}", gid, cid),
        format!("poker_pot_{}_{}", gid, cid),
        format!("poker_folded_{}_{}", gid, cid),
        format!("poker_candiscard_{}_{}", gid, cid),
        format!("poker_state_{}_{}", gid, cid),
        format!("poker_user_bet_{}_{}_*", gid, cid),
        format!("deck_poker_{}_{}", gid, cid),
    ];

    for pattern in patterns {
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query(&mut conn)
            .unwrap_or_default();
        for key in keys {
            let _: Result<(), _> = redis::cmd("DEL").arg(key).query(&mut conn);
        }
    }

    Ok(())
}
