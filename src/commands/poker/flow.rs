use std::time::Duration;

use serenity::{
    model::prelude::{ChannelId, GuildId, MessageId, UserId},
    prelude::Context,
};
use tracing::{info, warn};

use crate::{
    commands::poker::{
        bot,
        hand_evaluator::{self},
        session::{self, is_bot, Phase, PokerGameState},
        ui,
    },
    errors::GenericError,
    redis::{decks::draw_card, users},
    utils::deck::card_to_int,
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

    if state.players.is_empty() {
        cid.send_message(&ctx.http, |m| m.content("No players in the lobby."))
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
        session::delete_state(gid, cid)
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
        return Ok(());
    }

    if state.players.len() == 1 {
        state.add_bot();
        cid.send_message(&ctx.http, |m| {
            m.content("Only one player joined. A bot opponent has been added to the table.")
        })
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    }

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

    // deal hole cards
    for uid in state.players.clone() {
        let c1 = draw_card(gid, cid, 0, 52).await.map_err(|e| GenericError::new(&e.to_string()))?;
        let c2 = draw_card(gid, cid, 0, 52).await.map_err(|e| GenericError::new(&e.to_string()))?;
        state.set_hole_cards(UserId::from(uid), vec![card_to_int(c1), card_to_int(c2)]);
    }

    // post blinds
    let len = state.players.len();
    // in heads-up the dealer is the small blind; otherwise dealer is the button
    let (sb_pos, bb_pos) = if len == 2 {
        (state.dealer_index, (state.dealer_index + 1) % len)
    } else {
        (
            (state.dealer_index + 1) % len,
            (state.dealer_index + 2) % len,
        )
    };
    let sb_uid = UserId::from(state.players[sb_pos]);
    let bb_uid = UserId::from(state.players[bb_pos]);

    if is_bot(sb_uid) {
        state.bot_balance = state.bot_balance.saturating_sub(state.small_blind);
    } else {
        users::user_add(sb_uid, gid, -(state.small_blind as i64))
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
    }
    state.place_bet(sb_uid, state.small_blind);

    if is_bot(bb_uid) {
        state.bot_balance = state.bot_balance.saturating_sub(state.big_blind);
    } else {
        users::user_add(bb_uid, gid, -(state.big_blind as i64))
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
    }
    state.place_bet(bb_uid, state.big_blind);
    state.current_bet = state.big_blind;

    // action starts after big blind
    state.current_player_index = (bb_pos + 1) % len;

    session::save_state(gid, cid, &state)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

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
            let previous_phase = state.phase.clone();
            state.advance_phase();
            let new_phase = state.phase.clone();

            // deal community cards
            if previous_phase == Phase::PreFlop && new_phase == Phase::Flop {
                let c1 = draw_card(gid, cid, 0, 52).await.map_err(|e| GenericError::new(&e.to_string()))?;
                let c2 = draw_card(gid, cid, 0, 52).await.map_err(|e| GenericError::new(&e.to_string()))?;
                let c3 = draw_card(gid, cid, 0, 52).await.map_err(|e| GenericError::new(&e.to_string()))?;
                state.add_community_card(card_to_int(c1));
                state.add_community_card(card_to_int(c2));
                state.add_community_card(card_to_int(c3));
            } else if previous_phase == Phase::Flop && new_phase == Phase::Turn {
                let c = draw_card(gid, cid, 0, 52).await.map_err(|e| GenericError::new(&e.to_string()))?;
                state.add_community_card(card_to_int(c));
            } else if previous_phase == Phase::Turn && new_phase == Phase::River {
                let c = draw_card(gid, cid, 0, 52).await.map_err(|e| GenericError::new(&e.to_string()))?;
                state.add_community_card(card_to_int(c));
            }

            // reset betting for new round
            state.current_bet = 0;
            state.round_bets.clear();
            state.acted_this_round.clear();
            state.current_player_index = (state.dealer_index + 1) % state.players.len();

            session::save_state(gid, cid, &state)
                .await
                .map_err(|e| GenericError::new(&e.to_string()))?;

            update_status_message(ctx, gid, cid, &state).await?;

            if new_phase == Phase::Showdown {
                run_showdown(ctx, gid, cid).await?;
                return Ok(());
            }

            cid.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    let phase_name = match new_phase {
                        Phase::Flop => "Flop",
                        Phase::Turn => "Turn",
                        Phase::River => "River",
                        _ => "",
                    };
                    e.title(format!("{} Dealt", phase_name))
                        .description(ui::format_community_cards(&state))
                })
            })
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;

            continue;
        }

        state.turn_timer_id += 1;
        let timer_id = state.turn_timer_id;
        session::save_state(gid, cid, &state)
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
        update_status_message(ctx, gid, cid, &state).await?;

        if let Some(uid) = state.current_player() {
            if bot::is_bot(uid) {
                bot::start_bot_turn_timer(ctx.clone(), gid, cid, timer_id);
            } else {
                send_action_prompt(ctx, gid, cid, uid).await?;
                start_turn_timer(ctx.clone(), gid, cid, uid, state.turn_seconds, timer_id);
            }
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
    let state = session::load_state(gid, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
        .ok_or(GenericError::new(&"No poker game found."))?;

    let embed = ui::create_action_prompt_embed(&state, uid);
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

fn start_turn_timer(ctx: Context, gid: GuildId, cid: ChannelId, uid: UserId, seconds: u64, timer_id: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(seconds)).await;
        if let Err(e) = handle_auto_fold(&ctx, gid, cid, uid, timer_id).await {
            warn!("auto fold failed: {}", e);
        }
    });
}

async fn handle_auto_fold(
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
        if current != uid {
            return Ok(());
        }
    } else {
        return Ok(());
    }

    info!("auto folding {} in {} {}", uid, gid, cid);

    let embed = ui::create_timeout_embed(uid);
    cid.send_message(&ctx.http, |m| m.set_embed(embed))
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

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
    state.advance_turn();
    session::save_state(gid, cid, &state)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    let embed = ui::create_action_result_embed(&state, uid, "folded.");
    cid.send_message(&ctx.http, |m| m.set_embed(embed))
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

    if bot::is_bot(uid) {
        if state.bot_balance < to_call {
            return Err(GenericError::new(&"Bot does not have enough chips to call."));
        }
        state.bot_balance -= to_call;
    } else {
        let bal = users::get_user_bal(uid, gid)
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;

        if (bal as u64) < to_call {
            return Err(GenericError::new(&"Not enough chips to call."));
        }

        users::user_add(uid, gid, -(to_call as i64))
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
    }

    state.place_bet(uid, to_call);
    state.mark_acted(uid);
    state.advance_turn();

    session::save_state(gid, cid, &state)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    let action_text = if to_call == 0 {
        "checked.".to_string()
    } else {
        format!("called the {} chip bet.", current_bet)
    };
    let embed = ui::create_action_result_embed(&state, uid, &action_text);
    cid.send_message(&ctx.http, |m| m.set_embed(embed))
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

    if to_pay == 0 {
        return Err(GenericError::new(&"Raise amount is too small."));
    }

    if bot::is_bot(uid) {
        if state.bot_balance < to_pay {
            return Err(GenericError::new(&"Bot does not have enough chips to raise."));
        }
        state.bot_balance -= to_pay;
    } else {
        let bal = users::get_user_bal(uid, gid)
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;

        if (bal as u64) < to_pay {
            return Err(GenericError::new(&"Not enough chips to raise."));
        }

        users::user_add(uid, gid, -(to_pay as i64))
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
    }

    state.place_bet(uid, to_pay);
    state.current_bet = new_bet;
    state.clear_acted_except(uid);
    state.mark_acted(uid);
    state.advance_turn();

    session::save_state(gid, cid, &state)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    let action_text = format!("raised by {}. Current bet is now {}.", raise_amount, new_bet);
    let embed = ui::create_action_result_embed(&state, uid, &action_text);
    cid.send_message(&ctx.http, |m| m.set_embed(embed))
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

    let (amount, to_pay) = if bot::is_bot(uid) {
        let current_bet = state.current_bet;
        let user_bet = state.player_bet(uid);
        let to_call = current_bet.saturating_sub(user_bet);
        let amount = state.bot_balance.saturating_sub(to_call);
        let to_pay = to_call + amount;
        (amount, to_pay)
    } else {
        let bal = users::get_user_bal(uid, gid)
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;

        if bal <= 0 {
            return Err(GenericError::new(&"No chips to go all in."));
        }

        let amount = bal as u64;
        let current_bet = state.current_bet;
        let user_bet = state.player_bet(uid);
        let to_pay = amount + current_bet.saturating_sub(user_bet);

        let actual_bal = users::get_user_bal(uid, gid)
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
        if (actual_bal as u64) < to_pay {
            return Err(GenericError::new(&"Not enough chips to go all in."));
        }

        users::user_add(uid, gid, -bal)
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;

        (amount, to_pay)
    };

    if bot::is_bot(uid) {
        if state.bot_balance < to_pay {
            return Err(GenericError::new(&"Bot does not have enough chips to go all in."));
        }
        state.bot_balance -= to_pay;
    }

    let new_bet = state.current_bet + amount;
    state.place_bet(uid, to_pay);
    state.current_bet = new_bet;
    state.clear_acted_except(uid);
    state.mark_acted(uid);
    state.advance_turn();

    session::save_state(gid, cid, &state)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    let action_text = format!("went all in with {} chips!", to_pay);
    let embed = ui::create_action_result_embed(&state, uid, &action_text);
    cid.send_message(&ctx.http, |m| m.set_embed(embed))
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    update_status_message(ctx, gid, cid, &state).await?;
    start_turn(ctx, gid, cid).await?;
    Ok(())
}

async fn run_showdown(ctx: &Context, gid: GuildId, cid: ChannelId) -> Result<(), GenericError> {
    let mut state = session::load_state(gid, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
        .ok_or(GenericError::new(&"No poker game found."))?;

    let active: Vec<UserId> = state.active_players();
    if active.is_empty() {
        return Ok(());
    }

    if active.len() == 1 {
        // everyone else folded
        let winner = active[0];
        let pot = state.pot;
        if is_bot(winner) {
            state.bot_balance += pot;
        } else {
            users::user_add(winner, gid, pot as i64)
                .await
                .map_err(|e| GenericError::new(&e.to_string()))?;
        }

        cid.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Showdown")
                    .description(format!("{} wins the pot of {} chips!", ui::player_name(winner), pot))
            })
        })
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

        session::delete_state(gid, cid)
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
        return Ok(());
    }

    // evaluate hands
    let mut ranked: Vec<(UserId, Vec<u8>)> = active
        .iter()
        .map(|&uid| {
            let cards = state.all_cards_for_player(uid);
            let rank = hand_evaluator::evaluate_seven(&cards);
            (uid, rank)
        })
        .collect();

    ranked.sort_by(|a, b| b.1.cmp(&a.1));

    let best_rank = ranked[0].1.clone();
    let winners: Vec<UserId> = ranked
        .iter()
        .take_while(|(_, r)| *r == best_rank)
        .map(|(u, _)| *u)
        .collect();

    let pot = state.pot;
    let share = pot / winners.len() as u64;
    let remainder = pot % winners.len() as u64;

    for (i, &winner) in winners.iter().enumerate() {
        let amount = share + if i < remainder as usize { 1 } else { 0 };
        if is_bot(winner) {
            state.bot_balance += amount;
        } else {
            users::user_add(winner, gid, amount as i64)
                .await
                .map_err(|e| GenericError::new(&e.to_string()))?;
        }
    }

    // reveal all active hands
    let hand_desc = active
        .iter()
        .map(|&u| {
            let hole = state.hole_cards_eval(u);
            let hole_str = hole
                .iter()
                .map(|c| hand_evaluator::card_to_emoji(*c))
                .collect::<Vec<_>>()
                .join(" ");
            let rank = hand_evaluator::evaluate_seven(&state.all_cards_for_player(u));
            format!(
                "{}: {} ({})",
                ui::player_name(u),
                hole_str,
                hand_evaluator::rank_to_string(&rank)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let winners_str = winners
        .iter()
        .map(|&u| ui::player_name(u))
        .collect::<Vec<_>>()
        .join(", ");

    let embed = ui::create_showdown_embed(
        &winners
            .iter()
            .map(|&u| (u, hand_evaluator::rank_to_string(&best_rank)))
            .collect::<Vec<_>>(),
        pot,
    );

    cid.send_message(&ctx.http, |m| {
        m.set_embed(embed).content(format!(
            "Showdown!\n{}\n\nCommunity cards: {}\n\n{} wins the pot of {} chips!",
            hand_desc,
            ui::format_community_cards(&state),
            winners_str,
            pot
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
