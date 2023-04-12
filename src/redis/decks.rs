use serenity::model::prelude::ChannelId;
use tracing::log::warn;

use crate::utils::{card_to_int, game_id_to_name, Deck};

use super::get_conn;

//set the deck in redis
pub fn set_deck(cid: ChannelId, deck: Deck, game_id: u8) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = match get_conn() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let mut deck_local = deck.clone();
    let mut deck_vec = deck_local.deck;
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

pub fn get_deck(cid: ChannelId) {}

pub fn shuffle_deck(cid: ChannelId) {}

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
