use core::time;
use std::thread;

use redis::RedisError;
use serenity::model::prelude::ChannelId;
use tracing::{info, trace};

use crate::{
    constants::ROULETTE_EXPIRE_TIME_SECONDS,
    utils::roulette::{get_spin, SpinResult},
};

use super::get_db_link;

//check to see if there is a table
pub fn table_exists(id: ChannelId) -> Result<bool, RedisError> {
    let client = match redis::Client::open(get_db_link()) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let mut conn: redis::Connection = match client.get_connection() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    match redis::cmd("EXISTS")
        .arg(format!("roulette_{}", id.0))
        .query::<u8>(&mut conn)
    {
        Ok(e) => Ok(if e == 1 { true } else { false }),
        Err(e) => Err(e),
    }
}

//activate table mutex
pub fn activate_table(id: ChannelId) -> Result<(), RedisError> {
    let client = match redis::Client::open(get_db_link()) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let mut conn: redis::Connection = match client.get_connection() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    match redis::cmd("SET")
        .arg(format!("roulette_{}", id.0))
        .arg("1")
        .arg("EX")
        .arg(ROULETTE_EXPIRE_TIME_SECONDS)
        .query::<()>(&mut conn)
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

//spin the table
pub async fn spin_table(id: ChannelId) -> Result<Option<SpinResult>, RedisError> {
    if !match table_exists(id) {
        Ok(e) => {
            info!("table active: {}", e);
            e
        }
        Err(e) => return Err(e),
    } {
        match activate_table(id) {
            Ok(_) => {}
            Err(e) => return Err(e),
        };
        tokio::time::sleep(time::Duration::from_secs(ROULETTE_EXPIRE_TIME_SECONDS)).await;
        return Ok(Some(get_spin()));
    }

    Ok(None)
}
