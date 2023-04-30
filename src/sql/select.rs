use super::{get_db_link, statements::GET_ROULETTE_BETS, structs::BetResult};
use mysql_async::{prelude::Queryable, Pool};

use tracing::trace;

// get all roulette bets and put them into a vector
pub async fn get_all_bets(id: u64) -> Result<Vec<BetResult>, mysql_async::Error> {
    let url = get_db_link().await;

    let pool = Pool::new(url.as_str());

    let mut conn = pool.get_conn().await?;

    trace!("Getting all bets for roulette table {}", id);

    match conn
        .query_map(
            format!("{}{}", GET_ROULETTE_BETS, id),
            |(amount, user_id, bet_type, specific_bet)| BetResult {
                net: amount,
                user_id,
                bet_type,
                specific_bet,
            },
        )
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => Err(e),
    }
}
