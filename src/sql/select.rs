use super::{get_pool, statements, structs::BetResult};
use mysql_async::{params, prelude::Queryable};

use tracing::trace;

// get all roulette bets for a single channel and put them into a vector
pub async fn get_all_bets(id: u64) -> Result<Vec<BetResult>, mysql_async::Error> {
    let mut conn = get_pool().get_conn().await?;

    trace!("Getting all bets for roulette table {}", id);

    conn.exec_map(
        statements::GET_ROULETTE_BETS,
        params! {
            "channel_id" => id,
        },
        |(amount, user_id, bet_type, specific_bet)| BetResult {
            net: amount,
            user_id,
            bet_type,
            specific_bet,
            channel_id: id,
            guild_id: 0,
        },
    )
    .await
}

// get every open roulette bet across all channels
pub async fn get_all_open_roulette_bets() -> Result<Vec<BetResult>, mysql_async::Error> {
    let mut conn = get_pool().get_conn().await?;

    conn.query_map(
        statements::GET_ALL_ROULETTE_BETS,
        |(amount, user_id, channel_id, guild_id, bet_type, specific_bet)| BetResult {
            net: amount,
            user_id,
            bet_type,
            specific_bet,
            channel_id,
            guild_id,
        },
    )
    .await
}
