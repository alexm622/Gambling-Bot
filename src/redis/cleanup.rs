//cleanup redis

use redis::RedisError;

use super::get_conn;

pub async fn get_poker_hands() -> Result<Vec<String>, RedisError> {
    let mut conn = get_conn().await?;

    let vec = redis::cmd("KEYS")
        .arg("poker_*")
        .query::<Vec<String>>(&mut conn)?;
    Ok(vec)
}

pub async fn remove_keys(keys: Vec<String>) -> Result<(), RedisError> {
    if keys.len() == 0 {
        return Ok(());
    }
    let mut conn = get_conn().await?;

    redis::cmd("DEL").arg(keys).query(&mut conn)?;

    Ok(())
}

pub async fn remove_all_poker_hands() -> Result<(), RedisError> {
    let hands = get_poker_hands().await?;
    Ok(remove_keys(hands).await?)
}

pub async fn get_all_decks() -> Result<Vec<String>, RedisError> {
    let mut conn = get_conn().await?;

    let vec = redis::cmd("KEYS")
        .arg("deck_*")
        .query::<Vec<String>>(&mut conn)?;
    Ok(vec)
}

pub async fn remove_all_decks() -> Result<(), RedisError> {
    Ok(remove_keys(get_all_decks().await?).await?)
}
