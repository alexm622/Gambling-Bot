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
    let state = session::load_state_or_err(gid, cid).await?;

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

    let (action, amount) = decide_action(&state, &mut rand::thread_rng());

    // small delay so it feels like the bot is "thinking"
    tokio::time::sleep(Duration::from_secs(2)).await;

    if let Err(e) = flow::handle_action(ctx, gid, cid, uid, action, amount).await {
        warn!("bot action failed: {}", e);
        // fallback to fold
        let _ = flow::handle_action(ctx, gid, cid, uid, PlayerAction::Fold, None).await;
    }

    Ok(())
}

fn decide_action<R: Rng>(state: &PokerGameState, rng: &mut R) -> (PlayerAction, Option<u64>) {
    let bot_id = bot_user();
    let user_bet = state.player_bet(bot_id);
    let to_call = state.current_bet.saturating_sub(user_bet);
    let balance = state.bot_balance;

    if to_call > balance {
        // cannot afford to call; fold or shove what's left as a valid all-in
        if balance > 0 && rng.gen_range(0..100) < 30 {
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

fn schedule_bot_turn(
    ctx: Context,
    gid: GuildId,
    cid: ChannelId,
    uid: UserId,
    seconds: u64,
    timer_id: u64,
) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    fn state_with_bot(bot_balance: u64, current_bet: u64, bot_bet: u64) -> PokerGameState {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_bot();
        state.bot_balance = bot_balance;
        state.current_bet = current_bet;
        if bot_bet > 0 {
            state.round_bets.insert(BOT_USER_ID, bot_bet);
        }
        state
    }

    #[test]
    fn broke_bot_facing_bet_always_folds() {
        let state = state_with_bot(0, 100, 0);
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..100 {
            let (action, _) = decide_action(&state, &mut rng);
            assert_eq!(action, PlayerAction::Fold);
        }
    }

    #[test]
    fn short_stack_only_folds_or_shoves() {
        let state = state_with_bot(30, 100, 0);
        let mut rng = StdRng::seed_from_u64(7);
        for _ in 0..200 {
            let (action, _) = decide_action(&state, &mut rng);
            assert!(matches!(action, PlayerAction::Fold | PlayerAction::AllIn));
        }
    }

    #[test]
    fn no_bet_and_no_chips_always_checks() {
        let state = state_with_bot(0, 0, 0);
        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..100 {
            let (action, _) = decide_action(&state, &mut rng);
            assert_eq!(action, PlayerAction::CheckCall);
        }
    }

    #[test]
    fn never_raises_above_balance() {
        // only 60 chips: a raise of 50 on top of a 30 call is unaffordable
        let state = state_with_bot(60, 30, 0);
        let mut rng = StdRng::seed_from_u64(99);
        for _ in 0..200 {
            let (action, amount) = decide_action(&state, &mut rng);
            if action == PlayerAction::Raise {
                let raise = amount.expect("raise must carry an amount");
                assert!(30 + raise <= 60, "raise exceeds bot balance");
            }
        }
    }

    #[test]
    fn decisions_are_eventually_varied() {
        // sanity check the bot isn't stuck on a single action with chips and no bet
        let state = state_with_bot(1000, 0, 0);
        let mut rng = StdRng::seed_from_u64(1234);
        let mut saw_check = false;
        let mut saw_raise = false;
        for _ in 0..500 {
            let (action, _) = decide_action(&state, &mut rng);
            saw_check |= action == PlayerAction::CheckCall;
            saw_raise |= action == PlayerAction::Raise;
            if saw_check && saw_raise {
                return;
            }
        }
        panic!(
            "bot never varied its action (check: {}, raise: {})",
            saw_check, saw_raise
        );
    }
}
