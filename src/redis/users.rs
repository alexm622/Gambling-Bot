//users

use redis::RedisError;
use serenity::model::prelude::UserId;
use tracing::log::warn;

use crate::sql::structs::BetResult;

use super::get_conn;

const STARTING_BAL: i64 = 10000;

//get the balance of user (uid)
pub async fn get_user_bal(id: UserId) -> Result<i64, RedisError> {
    let mut conn = match get_conn().await {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let key = format!("user_{}", id.0);

    let mut bal = match redis::cmd("GET").arg(key).query::<i64>(&mut conn) {
        Ok(v) => v,
        Err(e) => {
            warn!("error encountered");
            warn!("{}", e);
            0
        }
    };

    if bal == 0 {
        bal = match create_user(id).await {
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
pub async fn create_user(id: UserId) -> Result<(), RedisError> {
    let mut conn = match get_conn().await {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    match redis::cmd("SET")
        .arg(format!("user_{}", id.0))
        .arg(STARTING_BAL)
        .query::<()>(&mut conn)
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

//set the balance of a user
pub async fn set_bal(id: UserId, bal: i64) -> Result<(), RedisError> {
    let mut conn = match get_conn().await {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    match redis::cmd("SET")
        .arg(format!("user_{}", id.0))
        .arg(bal)
        .query::<()>(&mut conn)
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

//check if a user exists
pub async fn user_exists(id: UserId) -> Result<bool, RedisError> {
    let mut conn = match get_conn().await {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    match redis::cmd("EXISTS")
        .arg(format!("user_{}", id.0))
        .query::<u8>(&mut conn)
    {
        Ok(e) => Ok(if e == 1 { true } else { false }),
        Err(e) => Err(e),
    }
}

//add i64 to userid
pub async fn user_add(id: UserId, add: i64) -> Result<(), RedisError> {
    let bal: i64 = match get_user_bal(id).await {
        Ok(v) => v as i64 + add,
        Err(e) => return Err(e),
    };

    match set_bal(id, bal).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

//apply a net balance to the user
pub async fn apply_winnings(winnings: Vec<BetResult>) {
    for win in winnings {
        if win.net > 0 {
            match user_add(UserId::from(win.user_id), win.net).await {
                Ok(_) => {}
                Err(e) => {
                    warn!("unable to add to balance");
                    warn!("{}", e);
                }
            }
        }
    }
}
