//servers

use redis::RedisError;
use serenity::model::prelude::ChannelId;

use super::get_conn;

//channel exists (WIP)
pub async fn channel_exists(id: ChannelId) -> Result<bool, RedisError> {
    let mut conn = match get_conn().await {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    match redis::cmd("EXISTS")
        .arg(format!("channel_{}", id.0))
        .query::<u8>(&mut conn)
    {
        Ok(e) => Ok(if e == 1 { true } else { false }),
        Err(e) => Err(e),
    }
}
