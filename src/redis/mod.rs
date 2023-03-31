use tracing::info;

use crate::secrets::get_secret;

pub mod users;

pub async fn get_db_link() -> String {
    let ip = get_secret("REDIS_IP").await.value;
    return format!("redis://{}", ip);
}

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
