//cleanup when first launch
// TODO this will eventually pay out any stale bets

use redis::RedisError;
use tracing::info;

use crate::{
    errors::GenericError,
    redis::cleanup::{remove_all_decks, remove_all_poker_hands},
};

pub async fn cleanup() -> Result<(), GenericError> {
    match clean_redis().await {
        Ok(_) => Ok(()),
        Err(e) => Err(GenericError::new(&e.to_string())),
    }?;

    Ok(())
}

async fn clean_redis() -> Result<(), RedisError> {
    //cleanup poker

    info!("Cleaning Redis");

    //decks
    remove_all_decks().await?;
    //hands
    remove_all_poker_hands().await?;

    info!("Done cleaning redis!");

    Ok(())
}
