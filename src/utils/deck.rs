use rand::seq::SliceRandom;
use serde::Serialize;
use tracing::warn;

use crate::sql::structs::{Card, Suite};

#[derive(Serialize, Default, Debug, PartialEq, Eq, Clone)]
pub struct Deck {
    pub deck: Vec<(Card, Suite)>,
}

impl Deck {
    pub fn new() -> Self {
        Deck { deck: Vec::new() }
    }
}

//generate a brand new deck of size (size) and shuffle it
pub fn generate_deck(size: u8) -> Deck {
    let mut deck = Deck::new();
    for i in 0..size {
        deck.deck.push(int_to_card(i));
    }
    shuffle_deck(&mut deck);
    deck
}

//shuffle the deck (Fisher-Yates)
pub fn shuffle_deck(deck: &mut Deck) {
    deck.deck.shuffle(&mut rand::thread_rng());
}

//convert enum tuple to int
pub fn card_to_int(card: (Card, Suite)) -> u8 {
    let card_u8 = card.0 as u8;
    let suite_u8 = card.1 as u8;

    suite_u8 * 13 + card_u8
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
            warn!("invalid card value: {}", v);
            Card::ONE
        }
    };

    let suite = match suite_u8 {
        0 => Suite::DIAMONDS,
        1 => Suite::HEARTS,
        2 => Suite::CLUBS,
        3 => Suite::SPADES,
        v => {
            warn!("invalid suite value: {}", v);
            Suite::SPADES
        }
    };
    (card, suite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn card_int_roundtrip_all_52() {
        for i in 0..52u8 {
            assert_eq!(card_to_int(int_to_card(i)), i, "roundtrip failed for {}", i);
        }
    }

    #[test]
    fn int_to_card_suit_boundaries() {
        assert_eq!(int_to_card(0), (Card::ONE, Suite::DIAMONDS));
        assert_eq!(int_to_card(12), (Card::KING, Suite::DIAMONDS));
        assert_eq!(int_to_card(13), (Card::ONE, Suite::HEARTS));
        assert_eq!(int_to_card(25), (Card::KING, Suite::HEARTS));
        assert_eq!(int_to_card(26), (Card::ONE, Suite::CLUBS));
        assert_eq!(int_to_card(38), (Card::KING, Suite::CLUBS));
        assert_eq!(int_to_card(39), (Card::ONE, Suite::SPADES));
        assert_eq!(int_to_card(51), (Card::KING, Suite::SPADES));
    }

    #[test]
    fn generated_deck_has_52_unique_cards() {
        let deck = generate_deck(52);
        assert_eq!(deck.deck.len(), 52);
        let unique: HashSet<u8> = deck.deck.iter().map(|&c| card_to_int(c)).collect();
        assert_eq!(unique.len(), 52);
    }

    #[test]
    fn shuffle_preserves_cards() {
        let mut deck = generate_deck(52);
        let before: HashSet<u8> = deck.deck.iter().map(|&c| card_to_int(c)).collect();
        shuffle_deck(&mut deck);
        let after: HashSet<u8> = deck.deck.iter().map(|&c| card_to_int(c)).collect();
        assert_eq!(before, after);
        assert_eq!(deck.deck.len(), 52);
    }
}
