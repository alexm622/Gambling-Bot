//servers

use serenity::model::prelude::ChannelId;

use super::get_conn;

pub fn channel_exists(id: ChannelId) -> Result<bool, Box<dyn std::error::Error>> {
    let mut conn = match get_conn() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    match redis::cmd("EXISTS")
        .arg(format!("channel_{}", id.0))
        .query::<u8>(&mut conn)
    {
        Ok(e) => Ok(if e == 1 { true } else { false }),
        Err(e) => Err(Box::new(e)),
    }
}
