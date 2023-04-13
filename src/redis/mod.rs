use redis::RedisError;
use tracing::info;

use crate::secrets::get_secret;

pub mod channels;
pub mod cleanup;
pub mod decks;
pub mod poker;
pub mod users;

//games
pub mod roulette;

pub async fn get_db_link() -> String {
    let ip = get_secret("REDIS_IP").value;
    return format!("redis://{}", ip);
}
//test the connection to redis
pub async fn test_connection() -> Result<(), Box<dyn std::error::Error>> {
    info!("the db link is \"{}\"", get_db_link().await);
    let client = match redis::Client::open(get_db_link().await) {
        Ok(v) => v,
        Err(e) => return Err(Box::new(e)),
    };

    match client.get_connection() {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

//get the redis connection
pub async fn get_conn() -> Result<redis::Connection, RedisError> {
    let client = match redis::Client::open(get_db_link().await) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let conn: redis::Connection = match client.get_connection() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    Ok(conn)
}
