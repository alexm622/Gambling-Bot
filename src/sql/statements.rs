//statements

//creation
pub const CREATE_ROULETTE_TABLE: &str = "CREATE TABLE IF NOT EXISTS `roulette_bets`(
  `bet_id` bigint unsigned NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `amount` int unsigned NOT NULL,
  `user_id` bigint unsigned NOT NULL,
  `channel_id` bigint unsigned NOT NULL,
  `guild_id` bigint unsigned NOT NULL,
  `bet_type` tinyint NOT NULL,
  `specific_bet` int NULL
);";

pub const CREATE_TRANSACTIONS_TABLE: &str = "CREATE TABLE IF NOT EXISTS `transactions`(
  `transaction_id` bigint unsigned NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `user_id` bigint unsigned NOT NULL,
  `guild_id` bigint unsigned NOT NULL,
  `amount` bigint NOT NULL,
  `type` varchar(32) NOT NULL,
  `game` varchar(32) NULL,
  `timestamp` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX `idx_user_guild` (`user_id`, `guild_id`),
  INDEX `idx_timestamp` (`timestamp`)
);";

pub const CREATE_USER_STATS_TABLE: &str = "CREATE TABLE IF NOT EXISTS `user_stats`(
  `user_id` bigint unsigned NOT NULL,
  `guild_id` bigint unsigned NOT NULL,
  `total_wagered` bigint unsigned NOT NULL DEFAULT 0,
  `total_won` bigint unsigned NOT NULL DEFAULT 0,
  `total_lost` bigint unsigned NOT NULL DEFAULT 0,
  `net_profit` bigint NOT NULL DEFAULT 0,
  `games_played` bigint unsigned NOT NULL DEFAULT 0,
  `biggest_win` bigint unsigned NOT NULL DEFAULT 0,
  `current_streak` bigint NOT NULL DEFAULT 0,
  `favorite_game` varchar(32) NULL,
  PRIMARY KEY (`user_id`, `guild_id`)
);";

//inserts

pub const INSERT_ROULETTE_BET: &str = "INSERT INTO roulette_bets
    (amount, user_id,guild_id,channel_id,bet_type,specific_bet)
    VALUES (:amount,:user_id,:guild_id,:channel_id,:bet_type,:specific_bet);";

pub const INSERT_TRANSACTION: &str = "INSERT INTO transactions
    (user_id, guild_id, amount, type, game)
    VALUES (:user_id, :guild_id, :amount, :type, :game);";

//cleanup
pub const DROP_OLD_BETS: &str = "DELETE FROM roulette_bets WHERE channel_id = :channel_id;";

pub const DELETE_ALL_ROULETTE_BETS: &str = "DELETE FROM roulette_bets;";

//select
pub const GET_ROULETTE_BETS: &str =
    "SELECT amount, user_id, bet_type, specific_bet FROM roulette_bets WHERE channel_id = :channel_id;";

pub const GET_ALL_ROULETTE_BETS: &str =
    "SELECT amount, user_id, channel_id, guild_id, bet_type, specific_bet FROM roulette_bets;";

pub const GET_TRANSACTIONS: &str =
    "SELECT amount, type, game, timestamp FROM transactions WHERE user_id = :user_id AND guild_id = :guild_id ORDER BY timestamp DESC LIMIT :limit;";

pub const GET_USER_STATS: &str =
    "SELECT total_wagered, total_won, total_lost, net_profit, games_played, biggest_win, current_streak, favorite_game FROM user_stats WHERE user_id = :user_id AND guild_id = :guild_id;";

//upserts
pub const UPSERT_USER_STATS: &str = "INSERT INTO user_stats
    (user_id, guild_id, total_wagered, total_won, total_lost, net_profit, games_played, biggest_win, current_streak, favorite_game)
    VALUES (:user_id, :guild_id, :total_wagered, :total_won, :total_lost, :net_profit, :games_played, :biggest_win, :current_streak, :favorite_game)
    ON DUPLICATE KEY UPDATE
    total_wagered = VALUES(total_wagered),
    total_won = VALUES(total_won),
    total_lost = VALUES(total_lost),
    net_profit = VALUES(net_profit),
    games_played = VALUES(games_played),
    biggest_win = VALUES(biggest_win),
    current_streak = VALUES(current_streak),
    favorite_game = VALUES(favorite_game);";

pub const GET_BALANCE_LEADERBOARD: &str = "SELECT user_id, guild_id FROM user_stats ORDER BY net_profit DESC LIMIT :limit;";

pub const GET_WINS_LEADERBOARD: &str = "SELECT user_id, guild_id FROM user_stats ORDER BY total_won DESC LIMIT :limit;";

pub const GET_BIGGEST_WIN_LEADERBOARD: &str = "SELECT user_id, guild_id FROM user_stats ORDER BY biggest_win DESC LIMIT :limit;";
