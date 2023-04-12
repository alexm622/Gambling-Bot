use serde::{Deserialize, Serialize};
use tracing::{info, log::warn};

use crate::sql::structs::{Card, Suite};

pub mod poker;
pub mod roulette;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Deck {
    pub deck: Vec<(Card, Suite)>,
}

pub const SIZE_POKER: u8 = 52 * 4;

impl Deck {
    pub fn new() -> Self {
        info!("creating new deck");
        Deck { deck: Vec::new() }
    }
}

//generate a brand new deck of size (size) and shuffle it
pub fn generate_deck(size: u8) -> Deck {
    let mut deck = Deck::new();
    info!("attempting to generate a new deck");
    for i in 0..size {
        deck.deck.push(int_to_card(i % 52));
    }
    info!("deck created!");
    shuffle_deck(&mut deck);
    info!("done shuffling");
    return deck;
}

//shuffle the deck
pub fn shuffle_deck(deck: &mut Deck) {
    let mut new_vec: Vec<(Card, Suite)> = Vec::new();

    while !deck.deck.is_empty() {
        let index = rand::random::<u8>() % deck.deck.len() as u8;
        let card = deck.deck.remove(index as usize);

        new_vec.push(card.clone());
    }

    deck.deck = new_vec;
}

//convert enum tuple to int
pub fn card_to_int(card: (Card, Suite)) -> u8 {
    let card_u8 = card.0 as u8;
    let suite_u8 = card.1 as u8;

    return suite_u8 * 13 + card_u8;
}

//convert the int to a enum tuple
pub fn int_to_card(card_u8: u8) -> (Card, Suite) {
    let card_u8 = card_u8 % 13 + 1;
    let suite_u8 = card_u8 % 4;
    let card = match card_u8 {
        1 => Card::ONE,
        2 => Card::TWO,
        3 => Card::THREE,
        4 => Card::FOUR,
        5 => Card::FIVE,
        6 => Card::SIX,
        7 => Card::SEVEN,
        8 => Card::EIGHT,
        9 => Card::NINE,
        10 => Card::TEN,
        11 => Card::JACK,
        12 => Card::QUEEN,
        13 => Card::KING,
        v => {
            warn!("something went wrong");
            warn!("got value of: {} for card", v);
            Card::ONE
        }
    };

    let suite = match suite_u8 {
        0 => Suite::DIAMONDS,
        1 => Suite::HEARTS,
        2 => Suite::CLUBS,
        3 => Suite::SPADES,
        v => {
            warn!("Something went wrong");
            warn!("got value of {} for suite", v);
            Suite::SPADES
        }
    };
    return (card, suite);
}

//convert the gameid in to a name
pub fn game_id_to_name(game_id: u8) -> String {
    match game_id {
        0 => String::from("poker"),
        _ => String::from("unknown"),
    }
}
