//poker

use serenity::model::prelude::{ChannelId, UserId};
use tracing::info;

use crate::{
    sql::structs::PokerHand,
    utils::{card_to_int, int_to_card, poker::get_new_poker_hand},
};

use super::get_conn;

pub fn get_user_hand(cid: ChannelId, uid: UserId) -> Result<PokerHand, Box<dyn std::error::Error>> {
    let mut conn = match get_conn() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    info!("attemptng to get user {} hand", uid);
    let key_name = format!("poker_{}_{}", cid, uid);

    //check if empty
    let len = match redis::cmd("LLEN")
        .arg(key_name.clone())
        .query::<u8>(&mut conn)
    {
        Ok(v) => v,
        Err(e) => return Err(Box::new(e)),
    };

    if len == 0 {
        let hand = get_new_poker_hand(cid)?;
        push_poker_hand(hand, cid, uid)?;
        return Ok(hand);
    }

    let mut hand_primative = match redis::cmd("LRANGE")
        .arg(key_name)
        .arg(0)
        .arg(len)
        .query::<Vec<u8>>(&mut conn)
    {
        Ok(v) => v,
        Err(e) => return Err(Box::new(e)),
    };

    let hand: PokerHand = PokerHand {
        one: int_to_card(hand_primative.pop().unwrap()),
        two: int_to_card(hand_primative.pop().unwrap()),
        three: int_to_card(hand_primative.pop().unwrap()),
        four: int_to_card(hand_primative.pop().unwrap()),
        five: int_to_card(hand_primative.pop().unwrap()),
    };

    push_poker_hand(hand, cid, uid)?;
    Ok(hand)
}

pub fn push_poker_hand(
    hand: PokerHand,
    cid: ChannelId,
    uid: UserId,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = match get_conn() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let key_name = format!("poker_{}_{}", cid, uid);

    match redis::cmd("DEL")
        .arg(key_name.clone())
        .query::<()>(&mut conn)
    {
        Ok(_) => {}
        Err(e) => return Err(Box::new(e)),
    };

    match redis::cmd("LPUSH")
        .arg(key_name)
        .arg(card_to_int(hand.one))
        .arg(card_to_int(hand.two))
        .arg(card_to_int(hand.three))
        .arg(card_to_int(hand.four))
        .arg(card_to_int(hand.five))
        .query::<()>(&mut conn)
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}
