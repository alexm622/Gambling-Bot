use mysql_async::{from_row, prelude::Queryable, Pool};
use tracing::{info, log::warn};

use super::{get_db_link, statements::GET_ROULETTE_BETS, structs::BetResult};

pub async fn get_all_bets(id: u64) -> Result<Vec<BetResult>, mysql_async::Error> {
    let url = get_db_link().await;

    let pool = Pool::new(url.as_str());

    let mut conn = pool.get_conn().await?;

    let ret: Vec<BetResult> = Vec::new();

    let mut iter = conn
        .query_iter(format!("{}{}", GET_ROULETTE_BETS, id))
        .await
        .unwrap();

    drop(
        iter.for_each(|row| {
            let r: (i64, u64, u8, Option<u8>) = from_row(row);
            info!("{} {} {}", r.0, r.1, r.2);
        })
        .await,
    );

    Ok(ret)
}
