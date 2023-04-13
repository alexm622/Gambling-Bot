use redis::RedisError;
use serenity::model::prelude::ChannelId;

use crate::{
    sql::structs::{Card, Suite},
    utils::{card_to_int, game_id_to_name, generate_deck, int_to_card, Deck},
};

use super::get_conn;

//set the deck in redis
pub async fn set_deck(cid: ChannelId, deck: Deck, game_id: u8) -> Result<(), RedisError> {
    let deck_local = deck.clone();
    let deck_vec = deck_local.deck;
    clear_deck(cid, game_id).await?;
    for card in deck_vec {
        match push_card(cid, game_id, card_to_int(card)).await {
            Ok(_) => {}
            Err(e) => return Err(e),
        };
    }
    Ok(())
}

//push a single card to redis
pub async fn push_card(cid: ChannelId, game_id: u8, card_id: u8) -> Result<(), RedisError> {
    let mut conn = get_conn().await?;

    let name = game_id_to_name(game_id);
    let key_name = format!("deck_{}_{}", name, cid.0);

    redis::cmd("LPUSH")
        .arg(key_name)
        .arg(card_id)
        .query::<()>(&mut conn)
}

pub async fn draw_card(cid: ChannelId, game_id: u8, size: u8) -> Result<(Card, Suite), RedisError> {
    let mut conn = get_conn().await?;
    let name = game_id_to_name(game_id);
    let key_name = format!("deck_{}_{}", name, cid.0);

    //check if empty
    let len = redis::cmd("LLEN")
        .arg(key_name.clone())
        .query::<u8>(&mut conn)?;
    if len == 0 {
        //generate and push a deck
        let new_deck = generate_deck(size);
        set_deck(cid, new_deck, game_id).await?;
    }

    let card = int_to_card(redis::cmd("LPOP").arg(key_name).query::<u8>(&mut conn)?);

    Ok(card)
}

pub async fn get_deck(cid: ChannelId, game_id: u8) -> Result<Deck, RedisError> {
    let mut conn = get_conn().await?;
    let name = game_id_to_name(game_id);
    let key_name = format!("deck_{}_{}", name, cid.0);

    let len = redis::cmd("LLEN")
        .arg(key_name.clone())
        .query::<u8>(&mut conn)?;

    let deck_primative = redis::cmd("LRANGE")
        .arg(key_name)
        .arg(0)
        .arg(len)
        .query::<Vec<u8>>(&mut conn)?;

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
pub async fn clear_deck(cid: ChannelId, game_id: u8) -> Result<(), RedisError> {
    let mut conn = get_conn().await?;

    let name = game_id_to_name(game_id);
    let key_name = format!("deck_{}_{}", name, cid.0);

    match redis::cmd("del").arg(key_name).query::<()>(&mut conn) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

//check if deck exists
pub async fn deck_exists(cid: ChannelId, game_id: u8) -> Result<bool, RedisError> {
    let mut conn = get_conn().await?;

    let name = game_id_to_name(game_id);
    let key_name = format!("deck_{}_{}", name, cid.0);

    redis::cmd("exists").arg(key_name).query::<bool>(&mut conn)
}
