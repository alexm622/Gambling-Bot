//users

use redis::RedisError;
use serenity::model::prelude::{GuildId, UserId};
use tracing::log::warn;

use crate::sql::{structs::BetResult, transactions::log_transaction};

use super::get_conn;

pub const STARTING_BAL: i64 = 10000;

//get the balance of user (uid)
pub async fn get_user_bal(id: UserId, gid: GuildId) -> Result<i64, RedisError> {
    let mut conn = match get_conn().await {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let key = format!("user_{}_{}", id.0, gid.0);

    let mut bal = match redis::cmd("GET").arg(key).query::<i64>(&mut conn) {
        Ok(v) => v,
        Err(e) => {
            warn!("error encountered");
            warn!("{}", e);
            0
        }
    };

    if bal == 0 {
        bal = match create_user(id, gid).await {
            Ok(_) => STARTING_BAL,
            Err(_) => {
                warn!("something went wrong setting the balance of {}", id.0);
                0
            }
        };
    }

    Ok(bal)
}

//create a user in redis
pub async fn create_user(id: UserId, gid: GuildId) -> Result<(), RedisError> {
    let mut conn = match get_conn().await {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    match redis::cmd("SET")
        .arg(format!("user_{}_{}", id.0, gid.0))
        .arg(STARTING_BAL)
        .query::<()>(&mut conn)
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

//set the balance of a user
pub async fn set_bal(id: UserId, gid: GuildId, bal: i64) -> Result<(), RedisError> {
    let mut conn = match get_conn().await {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    match redis::cmd("SET")
        .arg(format!("user_{}_{}", id.0, gid.0))
        .arg(bal)
        .query::<()>(&mut conn)
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

//add i64 to userid
pub async fn user_add(id: UserId, gid: GuildId, add: i64) -> Result<(), RedisError> {
    let bal: i64 = match get_user_bal(id, gid).await {
        Ok(v) => v + add,
        Err(e) => return Err(e),
    };

    match set_bal(id, gid, bal).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

//apply a net balance to the user
pub async fn apply_winnings(winnings: Vec<BetResult>, gid: GuildId) {
    for win in winnings {
        if win.net > 0 {
            match user_add(UserId::from(win.user_id), gid, win.net).await {
                Ok(_) => {
                    if let Err(e) =
                        log_transaction(UserId::from(win.user_id), gid, win.net, "win", "roulette")
                            .await
                    {
                        warn!("unable to log win transaction: {}", e);
                    }
                }
                Err(e) => {
                    warn!("unable to add to balance");
                    warn!("{}", e);
                }
            }
        } else if win.net < 0 {
            if let Err(e) = log_transaction(
                UserId::from(win.user_id),
                gid,
                win.net,
                "loss",
                "roulette",
            )
            .await
            {
                warn!("unable to log loss transaction: {}", e);
            }
        }
    }
}
