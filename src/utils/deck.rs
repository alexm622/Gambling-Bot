use serde::Serialize;
use tracing::{info, warn};

use crate::sql::structs::{Card, Suite};

#[derive(Serialize, Default, Debug, PartialEq, Eq, Clone)]
pub struct Deck {
    pub deck: Vec<(Card, Suite)>,
}

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
    info!("size: {}", size);
    for i in 0..size {
        deck.deck.push(int_to_card(i));
    }
    info!("deck created!");

    deck.deck
        .clone()
        .iter()
        .for_each(|v| info!("card: {}{}", v.0, v.1));

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
    let face_u8 = card_u8 % 13;
    let suite_u8 = (card_u8 - face_u8) / 13;

    let card = match face_u8 {
        0 => Card::ONE,
        1 => Card::TWO,
        2 => Card::THREE,
        3 => Card::FOUR,
        4 => Card::FIVE,
        5 => Card::SIX,
        6 => Card::SEVEN,
        7 => Card::EIGHT,
        8 => Card::NINE,
        9 => Card::TEN,
        10 => Card::JACK,
        11 => Card::QUEEN,
        12 => Card::KING,
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
