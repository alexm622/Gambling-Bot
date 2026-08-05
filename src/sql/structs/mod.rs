//structs

use std::fmt;

use serde::{Deserialize, Serialize};
use serenity::model::prelude::{ChannelId, GuildId, UserId};

use crate::commands::roulette::roulette_bet::BettingTypesEnum;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct RouletteBet {
    pub amount: i64,
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub guild_id: GuildId,
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
    LOW = 4,
    HIGH = 5,
    SPECIFIC = 6,
    GREEN = 7,
    INVALID = 8,
}

impl BettingTypes {
    pub fn from_bettingtypeenum(bet_type: BettingTypesEnum) -> BettingTypes {
        match bet_type {
            BettingTypesEnum::Red => BettingTypes::RED,
            BettingTypesEnum::Black => BettingTypes::BLACK,
            BettingTypesEnum::Green => BettingTypes::GREEN,
            BettingTypesEnum::Even => BettingTypes::EVEN,
            BettingTypesEnum::Odd => BettingTypes::ODD,
            BettingTypesEnum::Low => BettingTypes::LOW,
            BettingTypesEnum::High => BettingTypes::HIGH,
            BettingTypesEnum::Specific(_) => BettingTypes::SPECIFIC,
            BettingTypesEnum::Invalid => BettingTypes::INVALID,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct BetResult {
    pub user_id: u64,
    pub net: i64,
    pub bet_type: u8,
    pub specific_bet: Option<u8>,
}

impl fmt::Display for BetResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "user_id: {}, net: {}, bet_type: {}{}",
            self.user_id,
            self.net,
            self.bet_type,
            match self.specific_bet {
                Some(s) => format!(", specific_bet: {}", s),
                _ => String::new(),
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Card {
    ONE = 0,
    TWO = 1,
    THREE = 2,
    FOUR = 3,
    FIVE = 4,
    SIX = 5,
    SEVEN = 6,
    EIGHT = 7,
    NINE = 8,
    TEN = 9,
    JACK = 10,
    QUEEN = 11,
    KING = 12,
}
impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Card::ONE => write!(f, "ONE"),
            Card::TWO => write!(f, "TWO"),
            Card::THREE => write!(f, "THREE"),
            Card::FOUR => write!(f, "FOUR"),
            Card::FIVE => write!(f, "FIVE"),
            Card::SIX => write!(f, "SIX"),
            Card::SEVEN => write!(f, "SEVEN"),
            Card::EIGHT => write!(f, "EIGHT"),
            Card::NINE => write!(f, "NINE"),
            Card::TEN => write!(f, "TEN"),
            Card::JACK => write!(f, "JACK"),
            Card::QUEEN => write!(f, "QUEEN"),
            Card::KING => write!(f, "KING"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Suite {
    DIAMONDS = 0,
    HEARTS = 1,
    CLUBS = 2,
    SPADES = 3,
}

impl fmt::Display for Suite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Suite::DIAMONDS => write!(f, "DIAMONDS"),
            Suite::HEARTS => write!(f, "HEARTS"),
            Suite::CLUBS => write!(f, "CLUBS"),
            Suite::SPADES => write!(f, "SPADES"),
        }
    }
}
