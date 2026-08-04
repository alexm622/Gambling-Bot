use std::time::Duration;

use rand::Rng;
use serenity::{
    model::prelude::{ChannelId, GuildId, UserId},
    prelude::Context,
};
use tracing::warn;

use crate::errors::GenericError;

use super::{
    flow::{self, PlayerAction},
    session::{self, PokerGameState, BOT_USER_ID},
};

pub fn is_bot(uid: UserId) -> bool {
    uid.0 == BOT_USER_ID
}

pub fn bot_user() -> UserId {
    UserId::from(BOT_USER_ID)
}

pub async fn bot_take_turn(
    ctx: &Context,
    gid: GuildId,
    cid: ChannelId,
    uid: UserId,
    timer_id: u64,
) -> Result<(), GenericError> {
    let state = session::load_state(gid, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
        .ok_or(GenericError::new(&"No poker game found."))?;

    if state.turn_timer_id != timer_id {
        return Ok(());
    }

    if let Some(current) = state.current_player() {
        if current != uid || !is_bot(uid) {
            return Ok(());
        }
    } else {
        return Ok(());
    }

    let (action, amount) = decide_action(&state);

    // small delay so it feels like the bot is "thinking"
    tokio::time::sleep(Duration::from_secs(2)).await;

    if let Err(e) = flow::handle_action(ctx, gid, cid, uid, action, amount).await {
        warn!("bot action failed: {}", e);
        // fallback to fold
        let _ = flow::handle_action(ctx, gid, cid, uid, PlayerAction::Fold, None).await;
    }

    Ok(())
}

fn decide_action(state: &PokerGameState) -> (PlayerAction, Option<u64>) {
    let bot_id = bot_user();
    let user_bet = state.player_bet(bot_id);
    let to_call = state.current_bet.saturating_sub(user_bet);
    let balance = state.bot_balance;
    let mut rng = rand::thread_rng();

    if to_call > balance {
        // cannot afford to call; fold or shove what's left as a valid all-in
        if rng.gen_range(0..100) < 30 && balance > 0 && balance >= to_call {
            return (PlayerAction::AllIn, None);
        }
        return (PlayerAction::Fold, None);
    }

    if to_call == 0 {
        // no bet to call
        let roll = rng.gen_range(0..100);
        if roll < 70 {
            (PlayerAction::CheckCall, None)
        } else if roll < 90 && balance >= 50 {
            (PlayerAction::Raise, Some(50))
        } else if balance > 0 {
            (PlayerAction::AllIn, None)
        } else {
            (PlayerAction::CheckCall, None)
        }
    } else {
        // facing a bet
        let roll = rng.gen_range(0..100);
        if roll < 60 {
            (PlayerAction::CheckCall, None)
        } else if roll < 85 && balance >= to_call.saturating_add(50) {
            (PlayerAction::Raise, Some(50))
        } else if roll < 95 && balance > to_call {
            (PlayerAction::AllIn, None)
        } else {
            (PlayerAction::Fold, None)
        }
    }
}

fn schedule_bot_turn(ctx: Context, gid: GuildId, cid: ChannelId, uid: UserId, seconds: u64, timer_id: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(seconds)).await;
        if let Err(e) = bot_take_turn(&ctx, gid, cid, uid, timer_id).await {
            warn!("bot turn failed: {}", e);
        }
    });
}

pub fn start_bot_turn_timer(ctx: Context, gid: GuildId, cid: ChannelId, timer_id: u64) {
    schedule_bot_turn(ctx, gid, cid, bot_user(), 2, timer_id);
}


