// Centralized balance helpers that update Redis and log every change to SQL.

use serenity::model::prelude::{GuildId, UserId};
use tracing::warn;

use crate::{
    errors::GenericError,
    redis::users::{get_user_bal, set_bal},
    sql::transactions::log_transaction,
};

/// Change a user's balance by `delta` and log the transaction.
/// `delta` is positive for gains and negative for losses.
/// Returns the new balance.
pub async fn change_balance(
    user_id: UserId,
    guild_id: GuildId,
    delta: i64,
    transaction_type: &str,
    game: &str,
) -> Result<i64, GenericError> {
    let current = get_user_bal(user_id, guild_id)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    let new = current.saturating_add(delta);
    set_balance(user_id, guild_id, new, transaction_type, game).await?;
    Ok(new)
}

/// Set a user's balance to a specific value and log the resulting delta.
pub async fn set_balance(
    user_id: UserId,
    guild_id: GuildId,
    new_balance: i64,
    transaction_type: &str,
    game: &str,
) -> Result<(), GenericError> {
    let current = get_user_bal(user_id, guild_id)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    let delta = new_balance.saturating_sub(current);
    set_bal(user_id, guild_id, new_balance)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    if let Err(e) = log_transaction(user_id, guild_id, delta, transaction_type, game).await {
        warn!("Failed to log transaction for {}: {}", user_id.0, e);
    }
    Ok(())
}

/// Reset a user's balance to the starting amount and log it.
pub async fn reset_balance(user_id: UserId, guild_id: GuildId) -> Result<i64, GenericError> {
    change_balance(
        user_id,
        guild_id,
        crate::redis::users::STARTING_BAL.saturating_sub(
            get_user_bal(user_id, guild_id)
                .await
                .map_err(|e| GenericError::new(&e.to_string()))?,
        ),
        "reset",
        "system",
    )
    .await
}
