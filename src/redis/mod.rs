use tracing::info;

use crate::secrets::get_secret;

pub mod users;

pub fn get_db_link() -> String {
    let ip = get_secret("REDIS_IP").value;
    return format!("redis://{}", ip);
}

pub fn test_connection() -> Result<(), Box<dyn std::error::Error>> {
    info!("the db link is \"{}\"", get_db_link());
    let client = match redis::Client::open(get_db_link()) {
        Ok(v) => v,
        Err(e) => return Err(Box::new(e)),
    };

    match client.get_connection() {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}
