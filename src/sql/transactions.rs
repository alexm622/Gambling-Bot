// Transaction logging and user stats helpers.

use mysql_async::{params, prelude::Queryable};
use serenity::model::prelude::{GuildId, UserId};

use super::{get_pool, statements};

#[derive(Debug, Clone)]
pub struct Transaction {
    pub amount: i64,
    pub transaction_type: String,
    pub game: Option<String>,
    pub timestamp: String,
}

/// Log a balance change in the SQL transactions table.
/// `amount` is positive for gains and negative for losses.
pub async fn log_transaction(
    user_id: UserId,
    guild_id: GuildId,
    amount: i64,
    transaction_type: &str,
    game: &str,
) -> Result<(), mysql_async::Error> {
    let mut conn = get_pool().get_conn().await?;

    conn.exec_drop(
        statements::INSERT_TRANSACTION,
        params! {
            "user_id" => user_id.0,
            "guild_id" => guild_id.0,
            "amount" => amount,
            "type" => transaction_type,
            "game" => game,
        },
    )
    .await
}

pub async fn get_transactions(
    user_id: UserId,
    guild_id: GuildId,
    limit: u64,
) -> Result<Vec<Transaction>, mysql_async::Error> {
    let mut conn = get_pool().get_conn().await?;

    conn.exec_map(
        statements::GET_TRANSACTIONS,
        params! {
            "user_id" => user_id.0,
            "guild_id" => guild_id.0,
            "limit" => limit,
        },
        |(amount, transaction_type, game, timestamp)| Transaction {
            amount,
            transaction_type,
            game,
            timestamp,
        },
    )
    .await
}

#[derive(Debug, Clone, Default)]
pub struct UserStats {
    pub total_wagered: u64,
    pub total_won: u64,
    pub total_lost: u64,
    pub net_profit: i64,
    pub games_played: u64,
    pub biggest_win: u64,
    pub current_streak: i64,
    pub favorite_game: Option<String>,
}

pub async fn get_user_stats(
    user_id: UserId,
    guild_id: GuildId,
) -> Result<Option<UserStats>, mysql_async::Error> {
    let mut conn = get_pool().get_conn().await?;

    let rows: Vec<UserStats> = conn
        .exec_map(
            statements::GET_USER_STATS,
            params! {
                "user_id" => user_id.0,
                "guild_id" => guild_id.0,
            },
            |(
                total_wagered,
                total_won,
                total_lost,
                net_profit,
                games_played,
                biggest_win,
                current_streak,
                favorite_game,
            )| UserStats {
                total_wagered,
                total_won,
                total_lost,
                net_profit,
                games_played,
                biggest_win,
                current_streak,
                favorite_game,
            },
        )
        .await?;

    Ok(rows.into_iter().next())
}

pub async fn upsert_user_stats(
    user_id: UserId,
    guild_id: GuildId,
    stats: &UserStats,
) -> Result<(), mysql_async::Error> {
    let mut conn = get_pool().get_conn().await?;

    conn.exec_drop(
        statements::UPSERT_USER_STATS,
        params! {
            "user_id" => user_id.0,
            "guild_id" => guild_id.0,
            "total_wagered" => stats.total_wagered,
            "total_won" => stats.total_won,
            "total_lost" => stats.total_lost,
            "net_profit" => stats.net_profit,
            "games_played" => stats.games_played,
            "biggest_win" => stats.biggest_win,
            "current_streak" => stats.current_streak,
            "favorite_game" => stats.favorite_game.as_ref(),
        },
    )
    .await
}

/// Read a leaderboard by category. Returns (user_id, guild_id) tuples.
pub async fn get_leaderboard(
    category: &str,
    limit: u64,
) -> Result<Vec<(u64, u64)>, mysql_async::Error> {
    let mut conn = get_pool().get_conn().await?;

    let statement = match category {
        "wins" => statements::GET_WINS_LEADERBOARD,
        "biggest" => statements::GET_BIGGEST_WIN_LEADERBOARD,
        _ => statements::GET_BALANCE_LEADERBOARD,
    };

    conn.exec_map(
        statement,
        params! {
            "limit" => limit,
        },
        |(user_id, guild_id)| (user_id, guild_id),
    )
    .await
}
