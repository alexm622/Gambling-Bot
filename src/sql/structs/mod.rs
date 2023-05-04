//structs

use std::fmt;

use serde::{Deserialize, Serialize};
use serenity::model::prelude::{ChannelId, UserId, GuildId};

use crate::{utils::card_ascii::{BLACK_CARDS, RED_CARDS, SUITES}, commands::roulette::roulette_bet::BettingTypesEnum, redis::decks::draw_card};

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
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]

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

impl PokerHand {
    pub fn emoji_vec(&self) -> Vec<(String, String)>{
        let mut emojis: Vec<(String, String)> = Vec::new();
        
        let mut cs = get_card_suite(self.one);
        emojis.push(cs);

        cs = get_card_suite(self.two);
        emojis.push(cs);

        cs = get_card_suite(self.three);
        emojis.push(cs);

        cs = get_card_suite(self.four);
        emojis.push(cs);

        cs = get_card_suite(self.five);
        emojis.push(cs);

        return emojis;
    }

    pub async fn discard(&mut self,cards: String, uid:UserId, gid: GuildId, cid: ChannelId) -> Result<(), String>{

        //go through the cards and discard them

        for c in cards.chars(){
            let card = match c.to_digit(10){
                Some(n) => n,
                None => return Err(String::from("Invalid card")),
            };

            //see if anything is being discarded
            if card == 0{
                continue;
            }

            //using draw_card to get the card
            let new_card = match draw_card(gid, cid, 0, 1).await{
                Ok(c) => c,
                Err(e) => return Err(e.to_string()),
            };

            //replace the card
            match card{
                1 => self.one = new_card,
                2 => self.two = new_card,
                3 => self.three = new_card,
                4 => self.four = new_card,
                5 => self.five = new_card,
                _ => return Err(String::from("Invalid card")),
            }
        }
        Ok(())
    }
}

pub fn poker_hand_to_emojis(hand: PokerHand) -> String {
    let mut cards = String::new();
    let mut suites = String::new();

    


    //set the stuff
    let mut cs = get_card_suite(hand.one);
    cards = format!("{}{}", cards, cs.0);
    suites = format!("{}{}", suites, cs.1);

    cs = get_card_suite(hand.two);
    cards = format!("{}{}", cards, cs.0);
    suites = format!("{}{}", suites, cs.1);

    cs = get_card_suite(hand.three);
    cards = format!("{}{}", cards, cs.0);
    suites = format!("{}{}", suites, cs.1);

    cs = get_card_suite(hand.four);
    cards = format!("{}{}", cards, cs.0);
    suites = format!("{}{}", suites, cs.1);

    cs = get_card_suite(hand.five);
    cards = format!("{}{}", cards, cs.0);
    suites = format!("{}{}", suites, cs.1);

    return format!("{}\n{}", cards, suites);
}

pub fn get_card_suite(cards: (Card, Suite)) -> (String, String) {
    match cards.1 {
        Suite::DIAMONDS => {
            return (
                RED_CARDS[cards.0 as usize].to_string(),
                SUITES[cards.1 as usize].to_string(),
            );
        }
        Suite::HEARTS => {
            return (
                RED_CARDS[cards.0 as usize].to_string(),
                SUITES[cards.1 as usize].to_string(),
            );
        }
        Suite::SPADES => {
            return (
                BLACK_CARDS[cards.0 as usize].to_string(),
                SUITES[cards.1 as usize].to_string(),
            );
        }
        Suite::CLUBS => {
            return (
                BLACK_CARDS[cards.0 as usize].to_string(),
                SUITES[cards.1 as usize].to_string(),
            );
        }
    }
}
