//pokerutils

use serenity::model::prelude::ChannelId;

use crate::{redis::decks::draw_card, sql::structs::PokerHand};

use super::SIZE_POKER;

pub fn get_new_poker_hand(cid: ChannelId) -> Result<PokerHand, Box<dyn std::error::Error>> {
    let game_id = 0;
    let hand = PokerHand {
        one: draw_card(cid, game_id, SIZE_POKER)?,
        two: draw_card(cid, game_id, SIZE_POKER)?,
        three: draw_card(cid, game_id, SIZE_POKER)?,
        four: draw_card(cid, game_id, SIZE_POKER)?,
        five: draw_card(cid, game_id, SIZE_POKER)?,
    };

    Ok(hand)
}
