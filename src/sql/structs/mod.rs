//structs

use std::fmt;

use serde::{Deserialize, Serialize};
use serenity::model::prelude::{ChannelId, UserId};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct RouletteBet {
    pub amount: u32,
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub bet_type: BettingTypes,
    pub specific_bet: Option<u8>,
}

impl fmt::Display for RouletteBet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(amount: {}, user_id: {}, channel_id: {}, bet_type: {}{})",
            self.amount,
            self.user_id,
            self.channel_id,
            self.bet_type as u8,
            match self.specific_bet {
                Some(s) => format!(", specific_bet: {}", s),
                _ => String::new(),
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum BettingTypes {
    RED = 0,
    BLACK = 1,
    EVEN = 2,
    ODD = 3,
    SPECIFIC = 4,
}
