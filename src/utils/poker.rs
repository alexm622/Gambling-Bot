//pokerutils

use redis::RedisError;
use serenity::model::prelude::ChannelId;

use crate::{redis::decks::draw_card, sql::structs::PokerHand};

use super::SIZE_POKER;

pub async fn get_new_poker_hand(cid: ChannelId) -> Result<PokerHand, RedisError> {
    let game_id = 0;
    let hand = PokerHand {
        one: draw_card(cid, game_id, SIZE_POKER).await?,
        two: draw_card(cid, game_id, SIZE_POKER).await?,
        three: draw_card(cid, game_id, SIZE_POKER).await?,
        four: draw_card(cid, game_id, SIZE_POKER).await?,
        five: draw_card(cid, game_id, SIZE_POKER).await?,
    };

    Ok(hand)
}
