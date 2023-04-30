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

//inserts

pub const INSERT_ROULETTE_BET: &str = "INSERT INTO roulette_bets
    (amount, user_id,guild_id,channel_id,bet_type,specific_bet)
    VALUES (:amount,:user_id,:guild_id,:channel_id,:bet_type,:specific_bet);";

//cleanup
pub const DROP_OLD_BETS: &str = "DELETE FROM roulette_bets WHERE channel_id = ";

//select
pub const GET_ROULETTE_BETS: &str =
    "SELECT amount, user_id, bet_type, specific_bet FROM roulette_bets WHERE channel_id = ";
