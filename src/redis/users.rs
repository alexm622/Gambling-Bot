//users

use redis::ErrorKind;
use serenity::model::prelude::UserId;
use tracing::log::{info, warn};

use crate::sql::structs::BetResult;

use super::get_conn;

const STARTING_BAL: i64 = 10000;

pub fn get_user_bal(id: UserId) -> Result<i64, Box<dyn std::error::Error>> {
    let mut conn = match get_conn() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let bal = match redis::cmd("GET")
        .arg(format!("user_{}", id.0))
        .query(&mut conn)
    {
        Ok(v) => v,
        Err(e) => {
            warn!("error encountered");
            warn!("{}", e);
            if e.kind() == ErrorKind::TypeError {
                match create_user(id) {
                    Ok(_) => STARTING_BAL,
                    Err(_) => {
                        warn!("something went wrong setting the balance of {}", id.0);
                        warn!("{}", e);
                        0
                    }
                }
            } else {
                0
            }
        }
    };

    Ok(bal)
}

pub fn create_user(id: UserId) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = match get_conn() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    match redis::cmd("SET")
        .arg(format!("user_{}", id.0))
        .arg(STARTING_BAL)
        .query::<()>(&mut conn)
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn set_bal(id: UserId, bal: i64) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = match get_conn() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    match redis::cmd("SET")
        .arg(format!("user_{}", id.0))
        .arg(bal)
        .query::<()>(&mut conn)
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn user_exists(id: UserId) -> Result<bool, Box<dyn std::error::Error>> {
    let mut conn = match get_conn() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    match redis::cmd("EXISTS")
        .arg(format!("user_{}", id.0))
        .query::<u8>(&mut conn)
    {
        Ok(e) => Ok(if e == 1 { true } else { false }),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn user_add(id: UserId, add: i64) -> Result<(), Box<dyn std::error::Error>> {
    let bal: i64 = match get_user_bal(id) {
        Ok(v) => v as i64 + add,
        Err(e) => return Err(e),
    };

    match set_bal(id, bal) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn apply_winnings(winnings: Vec<BetResult>) {
    for win in winnings {
        if win.net > 0 {
            match user_add(UserId::from(win.user_id), win.net) {
                Ok(_) => {}
                Err(e) => {
                    warn!("unable to add to balance");
                    warn!("{}", e);
                }
            }
        }
    }
}
