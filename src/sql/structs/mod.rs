//structs

use std::fmt;

use serde::{Deserialize, Serialize};
use serenity::model::prelude::{ChannelId, UserId};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct RouletteBet {
    pub amount: i64,
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
            "user_id: {}, net: {}, bet_type{}{}",
            self.user_id,
            self.net,
            self.bet_type as u8,
            match self.specific_bet {
                Some(s) => format!(", specific_bet: {}", s),
                _ => String::new(),
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Card {
    ONE = 1,
    TWO = 2,
    THREE = 3,
    FOUR = 4,
    FIVE = 5,
    SIX = 6,
    SEVEN = 7,
    EIGHT = 8,
    NINE = 9,
    TEN = 10,
    JACK = 11,
    QUEEN = 12,
    KING = 13,
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

pub struct PokerHand {
    pub one: (Card, Suite),
    pub two: (Card, Suite),
    pub three: (Card, Suite),
    pub four: (Card, Suite),
    pub five: (Card, Suite),
}

impl fmt::Display for PokerHand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} of {} \n{} of {} \n{} of {} \n{} of {} \n{} of {} \n",
            self.one.0,
            self.one.1,
            self.two.0,
            self.two.1,
            self.three.0,
            self.three.1,
            self.four.0,
            self.four.1,
            self.five.0,
            self.five.1
        )
    }
}
