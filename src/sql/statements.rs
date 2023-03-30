//statements

//creation
pub const CREATE_ROULETTE_TABLE: &str = "CREATE TABLE `roulette_bets` IF NOT EXISTS(
  `bet_id` bigint unsigned NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `amount` int unsigned NOT NULL,
  `user_id` bigint unsigned NOT NULL,
  `channel_id` bigint unsigned NOT NULL,
  `bet_type` bit NOT NULL,
  `specific_bet` int NULL
);";

//inserts

pub const INSERT_ROULETTE_BET: &str = "INSERT INTO roulette_bets
    (amount, user_id,channel_id,bet_type,specific_bet)
    VALUES (:amount,:user_id,:channel_id,:bet_type,:specific_bet);";

//cleanup
