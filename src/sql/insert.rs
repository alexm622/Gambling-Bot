//insert into database

use mysql_async::{params, prelude::Queryable};

use crate::{
    errors::GenericError,
    redis::users::{get_user_bal, set_bal},
    sql::transactions::log_transaction,
};

use super::{get_pool, statements, structs::RouletteBet};

//insert a bet into roulette
pub async fn insert_roulette_bet(bet: RouletteBet) -> Result<(), GenericError> {
    let mut conn = match get_pool().get_conn().await {
        Ok(v) => v,
        Err(e) => return Err(GenericError::new(&e.to_string().clone())),
    };

    let balance: i64 = match get_user_bal(bet.user_id, bet.guild_id).await {
        Ok(v) => v,
        Err(e) => return Err(GenericError::new(&e.to_string().clone())),
    };
    if bet.amount > balance {
        return Err(GenericError::new(&"not enough balance to bet".to_owned()));
    }

    let new_balance = balance - bet.amount;

    let ret = match conn
        .exec_drop(
            statements::INSERT_ROULETTE_BET,
            params! {
                "amount" => bet.amount,
                "user_id" => bet.user_id.0,
                "channel_id" => bet.channel_id.0,
                "guild_id" => bet.guild_id.0,
                "bet_type" => bet.bet_type as u8,
                "specific_bet" => bet.specific_bet,
            },
        )
        .await
    {
        Ok(_) => {
            match set_bal(bet.user_id, bet.guild_id, new_balance).await {
                Ok(_) => Ok(()),
                Err(e) => return Err(GenericError::new(&e.to_string().clone())),
            }?;
            log_transaction(bet.user_id, bet.guild_id, -bet.amount, "bet", "roulette")
                .await
                .map_err(|e| GenericError::new(&e.to_string()))?;
            Ok(())
        }
        Err(e) => return Err(GenericError::new(&e.to_string().clone())),
    };
    ret
}
