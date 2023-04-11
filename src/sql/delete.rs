//delete stuff

use mysql_async::{prelude::Queryable, Pool};
use serenity::model::prelude::ChannelId;

use super::{get_db_link, statements::DROP_OLD_BETS};

pub async fn drop_old_bets(id: ChannelId) -> Result<(), mysql_async::Error> {
    let url = get_db_link().await;

    let pool = Pool::new(url.as_str());

    let mut conn = pool.get_conn().await?;

    conn.query_drop(format!("{}{}", DROP_OLD_BETS, id.0)).await
}
