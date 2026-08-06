//delete stuff

use mysql_async::{params, prelude::Queryable};
use serenity::model::prelude::ChannelId;

use super::{get_pool, statements};

//delete all old bets
pub async fn drop_old_bets(id: ChannelId) -> Result<(), mysql_async::Error> {
    let mut conn = get_pool().get_conn().await?;

    conn.exec_drop(
        statements::DROP_OLD_BETS,
        params! {
            "channel_id" => id.0,
        },
    )
    .await
}

//delete all roulette bets
pub async fn delete_all_roulette_bets() -> Result<(), mysql_async::Error> {
    let mut conn = get_pool().get_conn().await?;

    conn.query_drop(statements::DELETE_ALL_ROULETTE_BETS).await
}
