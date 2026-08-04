use std::time::Duration;

use serenity::{
    model::prelude::{ChannelId, GuildId, MessageId, UserId},
    prelude::Context,
};
use tracing::{info, warn};

use crate::{
    commands::poker::{
        session::{self, Phase, PokerGameState},
        ui,
    },
    errors::GenericError,
    redis::{poker as poker_redis, users},
    sql::structs::{poker_hand_to_emojis, PokerHand},
};

pub const LOBBY_SECONDS: u64 = 60;
pub const TURN_SECONDS: u64 = 30;

pub async fn start_lobby(
    ctx: &Context,
    gid: GuildId,
    cid: ChannelId,
    host: UserId,
) -> Result<(), GenericError> {
    let mut state = PokerGameState::new(host);
    state.lobby_seconds = LOBBY_SECONDS;
    state.turn_seconds = TURN_SECONDS;
    session::save_state(gid, cid, &state)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    let embed = ui::create_lobby_embed(&state, LOBBY_SECONDS);
    let buttons = ui::create_lobby_buttons(gid, cid);

    let message = cid
        .send_message(&ctx.http, |m| {
            m.set_embed(embed).components(|c| {
                for row in buttons {
                    c.add_action_row(row);
                }
                c
            })
        })
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    state.lobby_message_id = Some(message.id.0);
    session::save_state(gid, cid, &state)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(LOBBY_SECONDS)).await;
        if let Err(e) = auto_start_game(&ctx_clone, gid, cid).await {
            warn!("auto start game failed: {}", e);
        }
    });

    Ok(())
}

pub async fn handle_join(
    ctx: &Context,
    gid: GuildId,
    cid: ChannelId,
    uid: UserId,
) -> Result<(), GenericError> {
    let mut state = session::load_state(gid, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
        .ok_or(GenericError::new(&"No poker game found."))?;

    if state.phase != Phase::Lobby {
        return Err(GenericError::new(&"The game has already started."));
    }

    if !state.add_player(uid) {
        return Err(GenericError::new(&"You have already joined."));
    }

    session::save_state(gid, cid, &state)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    poker_redis::get_user_hand(gid, cid, uid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    poker_redis::clear_user_bet(gid, cid, uid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    update_lobby_message(ctx, gid, cid, &state).await?;
    Ok(())
}

pub async fn start_game_now(
    ctx: &Context,
    gid: GuildId,
    cid: ChannelId,
    _uid: UserId,
) -> Result<(), GenericError> {
    auto_start_game(ctx, gid, cid).await
}

async fn auto_start_game(ctx: &Context, gid: GuildId, cid: ChannelId) -> Result<(), GenericError> {
    let mut state = session::load_state(gid, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
        .ok_or(GenericError::new(&"No poker game found."))?;

    if state.phase != Phase::Lobby {
        return Ok(());
    }

    if state.players.len() < 2 {
        cid.send_message(&ctx.http, |m| {
            m.content("Not enough players to start poker.")
        })
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
        session::delete_state(gid, cid)
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
        return Ok(());
    }

    // disable lobby buttons
    if let Some(lobby_mid) = state.lobby_message_id {
        if let Ok(mut lobby_msg) = cid.message(&ctx.http, MessageId(lobby_mid)).await {
            let _ = lobby_msg
                .edit(&ctx.http, |m| {
                    m.components(|c| c)
                        .set_embed(ui::create_lobby_embed(&state, 0))
                })
                .await;
        }
    }

    state.advance_phase();
    session::save_state(gid, cid, &state)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    for uid in state.players.clone() {
        let _ = poker_redis::get_user_hand(gid, cid, UserId::from(uid))
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
    }

    let embed = ui::create_status_embed(&state);
    let message = cid
        .send_message(&ctx.http, |m| m.set_embed(embed))
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    state.status_message_id = Some(message.id.0);
    session::save_state(gid, cid, &state)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    info!("poker game started in {} {}", gid, cid);
    start_turn(ctx, gid, cid).await?;
    Ok(())
}

async fn start_turn(ctx: &Context, gid: GuildId, cid: ChannelId) -> Result<(), GenericError> {
    loop {
        let mut state = session::load_state(gid, cid)
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?
            .ok_or(GenericError::new(&"No poker game found."))?;

        state.skip_folded_players();

        if state.active_players().len() < 2 || state.betting_round_complete() {
            let phase = advance_phase_flow(ctx, gid, cid, &mut state).await?;
            match phase {
                Phase::Showdown | Phase::Finished => return Ok(()),
                Phase::Draw => {
                    cid.send_message(&ctx.http, |m| {
                        m.content("Draw phase! Use `/pdiscard` to replace cards (e.g. `1 3 5`) or `/phand` to view your hand.")
                    })
                    .await
                    .map_err(|e| GenericError::new(&e.to_string()))?;
                    schedule_draw_timer(ctx.clone(), gid, cid);
                    return Ok(());
                }
                Phase::SecondBet => {
                    continue;
                }
                _ => return Ok(()),
            }
        }

        session::save_state(gid, cid, &state)
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
        update_status_message(ctx, gid, cid, &state).await?;

        if let Some(uid) = state.current_player() {
            send_action_prompt(ctx, gid, cid, uid).await?;
            start_turn_timer(ctx.clone(), gid, cid, uid, state.turn_seconds);
        }
        return Ok(());
    }
}

async fn send_action_prompt(
    ctx: &Context,
    gid: GuildId,
    cid: ChannelId,
    uid: UserId,
) -> Result<(), GenericError> {
    let mut embed = serenity::builder::CreateEmbed::default();
    embed
        .title(format!("<@{}>'s Turn", uid.0))
        .description(format!(
            "It's <@{}>'s turn! They have {} seconds to act.",
            uid.0, TURN_SECONDS
        ))
        .color(serenity::utils::Colour::GOLD);

    let buttons = ui::create_action_buttons(gid, cid, uid);

    cid.send_message(&ctx.http, |m| {
        m.set_embed(embed).components(|c| {
            for row in buttons {
                c.add_action_row(row);
            }
            c
        })
    })
    .await
    .map_err(|e| GenericError::new(&e.to_string()))?;

    Ok(())
}

fn start_turn_timer(ctx: Context, gid: GuildId, cid: ChannelId, uid: UserId, seconds: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(seconds)).await;
        if let Err(e) = handle_auto_fold(&ctx, gid, cid, uid).await {
            warn!("auto fold failed: {}", e);
        }
    });
}

async fn handle_auto_fold(
    ctx: &Context,
    gid: GuildId,
    cid: ChannelId,
    uid: UserId,
) -> Result<(), GenericError> {
    let state = session::load_state(gid, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
        .ok_or(GenericError::new(&"No poker game found."))?;

    if let Some(current) = state.current_player() {
        if current != uid {
            return Ok(());
        }
    } else {
        return Ok(());
    }

    info!("auto folding {} in {} {}", uid, gid, cid);
    apply_fold(ctx, gid, cid, uid).await?;
    Ok(())
}

pub async fn handle_action(
    ctx: &Context,
    gid: GuildId,
    cid: ChannelId,
    uid: UserId,
    action: PlayerAction,
    amount: Option<u64>,
) -> Result<(), GenericError> {
    let state = session::load_state(gid, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
        .ok_or(GenericError::new(&"No poker game found."))?;

    if let Some(current) = state.current_player() {
        if current != uid {
            return Err(GenericError::new(&"It is not your turn."));
        }
    } else {
        return Err(GenericError::new(&"No active turn."));
    }

    if state.is_folded(uid) {
        return Err(GenericError::new(&"You have already folded."));
    }

    match action {
        PlayerAction::Fold => apply_fold(ctx, gid, cid, uid).await,
        PlayerAction::CheckCall => apply_check_call(ctx, gid, cid, uid).await,
        PlayerAction::Raise => {
            let raise = amount.unwrap_or(50);
            apply_raise(ctx, gid, cid, uid, raise).await
        }
        PlayerAction::AllIn => apply_all_in(ctx, gid, cid, uid).await,
    }
}

async fn apply_fold(
    ctx: &Context,
    gid: GuildId,
    cid: ChannelId,
    uid: UserId,
) -> Result<(), GenericError> {
    let mut state = session::load_state(gid, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
        .ok_or(GenericError::new(&"No poker game found."))?;

    state.fold(uid);
    state.mark_acted(uid);
    session::save_state(gid, cid, &state)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    cid.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Player Folded")
                .description(format!("<@{}> folded.", uid.0))
        })
    })
    .await
    .map_err(|e| GenericError::new(&e.to_string()))?;

    update_status_message(ctx, gid, cid, &state).await?;
    start_turn(ctx, gid, cid).await?;
    Ok(())
}

async fn apply_check_call(
    ctx: &Context,
    gid: GuildId,
    cid: ChannelId,
    uid: UserId,
) -> Result<(), GenericError> {
    let mut state = session::load_state(gid, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
        .ok_or(GenericError::new(&"No poker game found."))?;

    let current_bet = state.current_bet;
    let user_bet = state.player_bet(uid);
    let to_call = current_bet.saturating_sub(user_bet);

    let bal = users::get_user_bal(uid, gid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    if (bal as u64) < to_call {
        return Err(GenericError::new(&"Not enough chips to call."));
    }

    users::user_add(uid, gid, -(to_call as i64))
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    state.place_bet(uid, to_call);
    state.mark_acted(uid);

    session::save_state(gid, cid, &state)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    cid.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Player Action").description(format!(
                "<@{}> {}.",
                uid.0,
                if to_call == 0 { "checked" } else { "called" }
            ))
        })
    })
    .await
    .map_err(|e| GenericError::new(&e.to_string()))?;

    update_status_message(ctx, gid, cid, &state).await?;
    start_turn(ctx, gid, cid).await?;
    Ok(())
}

async fn apply_raise(
    ctx: &Context,
    gid: GuildId,
    cid: ChannelId,
    uid: UserId,
    raise_amount: u64,
) -> Result<(), GenericError> {
    let mut state = session::load_state(gid, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
        .ok_or(GenericError::new(&"No poker game found."))?;

    let current_bet = state.current_bet;
    let user_bet = state.player_bet(uid);
    let new_bet = current_bet + raise_amount;
    let to_pay = new_bet.saturating_sub(user_bet);

    let bal = users::get_user_bal(uid, gid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    if (bal as u64) < to_pay || to_pay == 0 {
        return Err(GenericError::new(&"Not enough chips to raise."));
    }

    users::user_add(uid, gid, -(to_pay as i64))
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    state.place_bet(uid, to_pay);
    state.current_bet = new_bet;
    state.clear_acted_except(uid);
    state.mark_acted(uid);

    session::save_state(gid, cid, &state)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    cid.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Player Action").description(format!(
                "<@{}> raised by {}. Current bet is now {}.",
                uid.0, raise_amount, new_bet
            ))
        })
    })
    .await
    .map_err(|e| GenericError::new(&e.to_string()))?;

    update_status_message(ctx, gid, cid, &state).await?;
    start_turn(ctx, gid, cid).await?;
    Ok(())
}

async fn apply_all_in(
    ctx: &Context,
    gid: GuildId,
    cid: ChannelId,
    uid: UserId,
) -> Result<(), GenericError> {
    let mut state = session::load_state(gid, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
        .ok_or(GenericError::new(&"No poker game found."))?;

    let bal = users::get_user_bal(uid, gid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    if bal <= 0 {
        return Err(GenericError::new(&"No chips to go all in."));
    }

    let amount = bal as u64;
    let current_bet = state.current_bet;
    let user_bet = state.player_bet(uid);
    let new_bet = current_bet + amount;
    let to_pay = amount + current_bet.saturating_sub(user_bet);

    let actual_bal = users::get_user_bal(uid, gid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    if (actual_bal as u64) < to_pay {
        return Err(GenericError::new(&"Not enough chips to go all in."));
    }

    users::user_add(uid, gid, -(to_pay as i64))
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    state.place_bet(uid, to_pay);
    state.current_bet = new_bet;
    state.clear_acted_except(uid);
    state.mark_acted(uid);

    session::save_state(gid, cid, &state)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    cid.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Player Action")
                .description(format!("<@{}> went all in with {} chips!", uid.0, to_pay))
        })
    })
    .await
    .map_err(|e| GenericError::new(&e.to_string()))?;

    update_status_message(ctx, gid, cid, &state).await?;
    start_turn(ctx, gid, cid).await?;
    Ok(())
}

async fn advance_phase_flow(
    ctx: &Context,
    gid: GuildId,
    cid: ChannelId,
    state: &mut PokerGameState,
) -> Result<Phase, GenericError> {
    if state.active_players().len() < 2 {
        state.phase = Phase::Showdown;
    } else {
        state.advance_phase();
    }

    session::save_state(gid, cid, state)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    update_status_message(ctx, gid, cid, state).await?;

    if state.phase == Phase::Showdown {
        run_showdown(ctx, gid, cid).await?;
    }

    Ok(state.phase.clone())
}

pub async fn advance_from_draw(
    ctx: &Context,
    gid: GuildId,
    cid: ChannelId,
) -> Result<(), GenericError> {
    let mut state = session::load_state(gid, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
        .ok_or(GenericError::new(&"No poker game found."))?;

    if state.phase != Phase::Draw {
        return Ok(());
    }

    advance_phase_flow(ctx, gid, cid, &mut state).await?;
    start_turn(ctx, gid, cid).await?;
    Ok(())
}

fn schedule_draw_timer(ctx: Context, gid: GuildId, cid: ChannelId) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(60)).await;
        if let Err(e) = advance_from_draw(&ctx, gid, cid).await {
            warn!("advance from draw failed: {}", e);
        }
    });
}

async fn run_showdown(ctx: &Context, gid: GuildId, cid: ChannelId) -> Result<(), GenericError> {
    let state = session::load_state(gid, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
        .ok_or(GenericError::new(&"No poker game found."))?;

    let active = state.active_players();
    if active.is_empty() {
        return Ok(());
    }

    let mut hands: Vec<(UserId, PokerHand)> = Vec::new();
    for uid in active.clone() {
        if let Ok(hand) = poker_redis::get_user_hand(gid, cid, uid).await {
            hands.push((uid, hand));
        }
    }

    let winner = hands.first().map(|(u, _)| *u).unwrap_or(active[0]);
    let pot = state.pot;

    users::user_add(winner, gid, pot as i64)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    let hand_desc = hands
        .iter()
        .map(|(u, h)| format!("<@{}>: {}", u.0, poker_hand_to_emojis(*h)))
        .collect::<Vec<_>>()
        .join("\n");

    let embed = ui::create_showdown_embed(&[(winner, "hand".to_string())], pot);
    cid.send_message(&ctx.http, |m| {
        m.set_embed(embed).content(format!(
            "Showdown!\n{}\n\n<@{}> wins the pot of {} chips!",
            hand_desc, winner.0, pot
        ))
    })
    .await
    .map_err(|e| GenericError::new(&e.to_string()))?;

    session::delete_state(gid, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    Ok(())
}

async fn update_lobby_message(
    ctx: &Context,
    gid: GuildId,
    cid: ChannelId,
    state: &PokerGameState,
) -> Result<(), GenericError> {
    if let Some(mid) = state.lobby_message_id {
        let mut message = cid
            .message(&ctx.http, MessageId(mid))
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;

        let remaining = state.lobby_seconds;
        let embed = ui::create_lobby_embed(state, remaining);
        message
            .edit(&ctx.http, |m| {
                m.set_embed(embed).components(|c| {
                    for row in ui::create_lobby_buttons(gid, cid) {
                        c.add_action_row(row);
                    }
                    c
                })
            })
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
    }
    Ok(())
}

async fn update_status_message(
    ctx: &Context,
    _gid: GuildId,
    cid: ChannelId,
    state: &PokerGameState,
) -> Result<(), GenericError> {
    if let Some(mid) = state.status_message_id {
        let mut message = cid
            .message(&ctx.http, MessageId(mid))
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;

        let embed = ui::create_status_embed(state);
        message
            .edit(&ctx.http, |m| m.set_embed(embed))
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAction {
    Fold,
    CheckCall,
    Raise,
    AllIn,
}

impl PlayerAction {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "fold" => Some(PlayerAction::Fold),
            "checkcall" => Some(PlayerAction::CheckCall),
            "raise" => Some(PlayerAction::Raise),
            "allin" => Some(PlayerAction::AllIn),
            _ => None,
        }
    }
}
