use serenity::model::prelude::ChannelId;
use tracing::log::warn;

use crate::{
    sql::structs::{Card, Suite},
    utils::{card_to_int, game_id_to_name, generate_deck, int_to_card, Deck},
};

use super::get_conn;

//set the deck in redis
pub fn set_deck(cid: ChannelId, deck: Deck, game_id: u8) -> Result<(), Box<dyn std::error::Error>> {
    let deck_local = deck.clone();
    let deck_vec = deck_local.deck;
    clear_deck(cid, game_id)?;
    for card in deck_vec {
        match push_card(cid, game_id, card_to_int(card)) {
            Ok(_) => {}
            Err(e) => {
                warn!("error occured");
                warn!("{}", e);
                return Err(e);
            }
        };
    }
    Ok(())
}

//push a single card to redis
pub fn push_card(
    cid: ChannelId,
    game_id: u8,
    card_id: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = match get_conn() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let name = game_id_to_name(game_id);
    let key_name = format!("deck_{}_{}", name, cid.0);

    match redis::cmd("LPUSH")
        .arg(key_name)
        .arg(card_id)
        .query::<()>(&mut conn)
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn draw_card(
    cid: ChannelId,
    game_id: u8,
    size: u8,
) -> Result<(Card, Suite), Box<dyn std::error::Error>> {
    let mut conn = match get_conn() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let name = game_id_to_name(game_id);
    let key_name = format!("deck_{}_{}", name, cid.0);

    //check if empty
    let len = match redis::cmd("LLEN")
        .arg(key_name.clone())
        .query::<u8>(&mut conn)
    {
        Ok(v) => v,
        Err(e) => return Err(Box::new(e)),
    };

    if len == 0 {
        //generate and push a deck
        let new_deck = generate_deck(size);
        set_deck(cid, new_deck, game_id)?;
    }

    let card = int_to_card(
        match redis::cmd("LPOP").arg(key_name).query::<u8>(&mut conn) {
            Ok(v) => v,
            Err(e) => return Err(Box::new(e)),
        },
    );

    Ok(card)
}

pub fn get_deck(cid: ChannelId, game_id: u8) -> Result<Deck, Box<dyn std::error::Error>> {
    let mut conn = match get_conn() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let name = game_id_to_name(game_id);
    let key_name = format!("deck_{}_{}", name, cid.0);

    let len = match redis::cmd("LLEN")
        .arg(key_name.clone())
        .query::<u8>(&mut conn)
    {
        Ok(v) => v,
        Err(e) => return Err(Box::new(e)),
    };

    let deck_primative = match redis::cmd("LRANGE")
        .arg(key_name)
        .arg(0)
        .arg(len)
        .query::<Vec<u8>>(&mut conn)
    {
        Ok(v) => v,
        Err(e) => return Err(Box::new(e)),
    };

    Ok(primative_to_deck(deck_primative))
}

pub fn primative_to_deck(deck_primative: Vec<u8>) -> Deck {
    let mut deck: Deck = Deck::new();

    for i in deck_primative {
        deck.deck.push(int_to_card(i));
    }

    return deck;
}

//delete a deck
pub fn clear_deck(cid: ChannelId, game_id: u8) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = match get_conn() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let name = game_id_to_name(game_id);
    let key_name = format!("deck_{}_{}", name, cid.0);

    match redis::cmd("del").arg(key_name).query::<()>(&mut conn) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

//check if deck exists
pub fn deck_exists(cid: ChannelId, game_id: u8) -> Result<bool, Box<dyn std::error::Error>> {
    let mut conn = match get_conn() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let name = game_id_to_name(game_id);
    let key_name = format!("deck_{}_{}", name, cid.0);

    match redis::cmd("exists").arg(key_name).query::<bool>(&mut conn) {
        Ok(e) => Ok(e),
        Err(e) => Err(Box::new(e)),
    }
}
