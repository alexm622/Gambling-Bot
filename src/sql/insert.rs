//insert into database

use mysql_async::{params, prelude::Queryable, Pool};

use super::{get_db_link, statements, structs::RouletteBet};

pub async fn insert_roulette_bet(bet: RouletteBet) -> Result<(), Box<dyn std::error::Error>> {
    let url = get_db_link().await;

    let pool = Pool::new(url.as_str());

    let mut conn = pool.get_conn().await?;

    match conn
        .exec_drop(
            statements::INSERT_ROULETTE_BET,
            params! {
                "amount" => bet.amount,
                "user_id" => bet.user_id.0,
                "channel_id" => bet.channel_id.0,
                "bet_type" => bet.bet_type as u8,
                "specific_bet" => bet.specific_bet,
            },
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}
