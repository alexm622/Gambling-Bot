//users

use redis::ErrorKind;
use serenity::model::prelude::UserId;
use tracing::log::warn;

use super::get_conn;

const STARTING_BAL: u64 = 10000;

pub fn get_user_bal(id: UserId) -> Result<u64, Box<dyn std::error::Error>> {
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

pub fn set_bal(id: UserId, bal: u64) -> Result<(), Box<dyn std::error::Error>> {
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
