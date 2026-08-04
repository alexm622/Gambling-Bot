// Game session / state machine for Texas Hold'em poker.

use std::collections::{HashMap, HashSet};

use redis::RedisError;
use serde::{Deserialize, Serialize};
use serenity::model::prelude::{ChannelId, GuildId, UserId};

use super::hand_evaluator::EvalCard;

pub const BOT_USER_ID: u64 = 999_999_999_999_999_999;

const DEFAULT_TURN_SECONDS: u64 = 30;
const DEFAULT_LOBBY_SECONDS: u64 = 60;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Lobby,
    PreFlop,
    Flop,
    Turn,
    River,
    Showdown,
    Finished,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PokerGameState {
    pub phase: Phase,
    pub players: Vec<u64>,
    pub current_player_index: usize,
    pub dealer_index: usize,
    pub pot: u64,
    pub current_bet: u64,
    pub round_bets: HashMap<u64, u64>,
    pub folded: HashSet<u64>,
    pub acted_this_round: HashSet<u64>,
    pub community_cards: Vec<u8>,
    pub hole_cards: HashMap<u64, Vec<u8>>,
    pub lobby_message_id: Option<u64>,
    pub status_message_id: Option<u64>,
    pub turn_seconds: u64,
    pub lobby_seconds: u64,
    pub has_bot: bool,
    pub bot_balance: u64,
    pub small_blind: u64,
    pub big_blind: u64,
    pub turn_timer_id: u64,
}

const DEFAULT_BOT_BALANCE: u64 = 1000;

impl PokerGameState {
    pub fn new(host: UserId) -> Self {
        Self {
            phase: Phase::Lobby,
            players: vec![host.0],
            current_player_index: 0,
            dealer_index: 0,
            pot: 0,
            current_bet: 0,
            round_bets: HashMap::new(),
            folded: HashSet::new(),
            acted_this_round: HashSet::new(),
            community_cards: Vec::new(),
            hole_cards: HashMap::new(),
            lobby_message_id: None,
            status_message_id: None,
            turn_seconds: DEFAULT_TURN_SECONDS,
            lobby_seconds: DEFAULT_LOBBY_SECONDS,
            has_bot: false,
            bot_balance: DEFAULT_BOT_BALANCE,
            small_blind: 25,
            big_blind: 50,
            turn_timer_id: 0,
        }
    }

    pub fn current_player(&self) -> Option<UserId> {
        self.players
            .get(self.current_player_index)
            .copied()
            .map(UserId::from)
    }

    pub fn add_player(&mut self, uid: UserId) -> bool {
        if self.players.contains(&uid.0) {
            return false;
        }
        self.players.push(uid.0);
        true
    }

    pub fn add_bot(&mut self) {
        if !self.players.contains(&BOT_USER_ID) {
            self.players.push(BOT_USER_ID);
            self.has_bot = true;
        }
    }

    pub fn remove_player(&mut self, uid: UserId) -> bool {
        if let Some(pos) = self.players.iter().position(|&id| id == uid.0) {
            self.players.remove(pos);
            self.folded.insert(uid.0);
            self.round_bets.remove(&uid.0);
            self.acted_this_round.insert(uid.0);
            self.hole_cards.remove(&uid.0);
            if self.current_player_index >= self.players.len() && !self.players.is_empty() {
                self.current_player_index = 0;
            }
            true
        } else {
            false
        }
    }

    pub fn is_folded(&self, uid: UserId) -> bool {
        self.folded.contains(&uid.0)
    }

    pub fn active_players(&self) -> Vec<UserId> {
        self.players
            .iter()
            .filter(|&&id| !self.folded.contains(&id))
            .copied()
            .map(UserId::from)
            .collect()
    }

    pub fn player_bet(&self, uid: UserId) -> u64 {
        *self.round_bets.get(&uid.0).unwrap_or(&0)
    }

    pub fn place_bet(&mut self, uid: UserId, amount: u64) {
        let current = self.player_bet(uid);
        self.round_bets.insert(uid.0, current + amount);
        self.pot += amount;
    }

    pub fn fold(&mut self, uid: UserId) {
        self.folded.insert(uid.0);
        self.acted_this_round.insert(uid.0);
    }

    pub fn mark_acted(&mut self, uid: UserId) {
        self.acted_this_round.insert(uid.0);
    }

    pub fn clear_acted_except(&mut self, uid: UserId) {
        self.acted_this_round.clear();
        self.acted_this_round.insert(uid.0);
    }

    pub fn advance_turn(&mut self) -> bool {
        let active = self.active_players();
        if active.len() < 2 {
            return false;
        }

        for _ in 0..active.len() {
            self.current_player_index = (self.current_player_index + 1) % self.players.len();
            if let Some(next) = self.current_player() {
                if self.is_folded(next) {
                    continue;
                }
                if !self.acted_this_round.contains(&next.0) {
                    return true;
                }
                let player_bet = self.player_bet(next);
                if player_bet < self.current_bet {
                    return true;
                }
            }
        }

        false
    }

    pub fn betting_round_complete(&self) -> bool {
        let active = self.active_players();
        if active.len() < 2 {
            return true;
        }
        for uid in active {
            let bet = self.player_bet(uid);
            if bet < self.current_bet {
                return false;
            }
            if !self.acted_this_round.contains(&uid.0) {
                return false;
            }
        }
        true
    }

    pub fn advance_phase(&mut self) {
        match self.phase {
            Phase::Lobby => {
                self.phase = Phase::PreFlop;
                self.current_bet = self.big_blind;
                self.round_bets.clear();
                self.acted_this_round.clear();
                self.community_cards.clear();
            }
            Phase::PreFlop => {
                self.phase = Phase::Flop;
                self.current_bet = 0;
                self.round_bets.clear();
                self.acted_this_round.clear();
            }
            Phase::Flop => {
                self.phase = Phase::Turn;
                self.current_bet = 0;
                self.round_bets.clear();
                self.acted_this_round.clear();
            }
            Phase::Turn => {
                self.phase = Phase::River;
                self.current_bet = 0;
                self.round_bets.clear();
                self.acted_this_round.clear();
            }
            Phase::River => {
                self.phase = Phase::Showdown;
                self.current_bet = 0;
            }
            _ => {}
        }
    }

    pub fn skip_folded_players(&mut self) {
        let len = self.players.len();
        if len == 0 {
            return;
        }
        for _ in 0..len {
            if let Some(current) = self.current_player() {
                if !self.is_folded(current) {
                    break;
                }
            }
            self.current_player_index = (self.current_player_index + 1) % len;
        }
    }

    pub fn hole_cards_eval(&self, uid: UserId) -> Vec<EvalCard> {
        self.hole_cards
            .get(&uid.0)
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|&c| super::hand_evaluator::card_tuple_to_eval(crate::utils::deck::int_to_card(c)))
            .collect()
    }

    pub fn community_cards_eval(&self) -> Vec<EvalCard> {
        self.community_cards
            .iter()
            .map(|&c| super::hand_evaluator::card_tuple_to_eval(crate::utils::deck::int_to_card(c)))
            .collect()
    }

    pub fn all_cards_for_player(&self, uid: UserId) -> Vec<EvalCard> {
        let mut all = self.hole_cards_eval(uid);
        all.extend(self.community_cards_eval());
        all
    }

    pub fn set_hole_cards(&mut self, uid: UserId, cards: Vec<u8>) {
        self.hole_cards.insert(uid.0, cards);
    }

    pub fn add_community_card(&mut self, card: u8) {
        self.community_cards.push(card);
    }
}

pub fn is_bot(uid: UserId) -> bool {
    uid.0 == BOT_USER_ID
}

pub fn state_key(gid: GuildId, cid: ChannelId) -> String {
    format!("poker_state_{}_{}", gid, cid)
}

pub async fn load_state(
    gid: GuildId,
    cid: ChannelId,
) -> Result<Option<PokerGameState>, RedisError> {
    let mut conn = crate::redis::get_conn().await?;
    let key = state_key(gid, cid);
    let raw: Option<String> = redis::cmd("GET").arg(key).query(&mut conn)?;
    match raw {
        Some(s) => match serde_json::from_str(&s) {
            Ok(state) => Ok(Some(state)),
            Err(_) => Ok(None),
        },
        None => Ok(None),
    }
}

pub async fn save_state(
    gid: GuildId,
    cid: ChannelId,
    state: &PokerGameState,
) -> Result<(), RedisError> {
    let mut conn = crate::redis::get_conn().await?;
    let key = state_key(gid, cid);
    let raw = serde_json::to_string(state).unwrap_or_default();
    redis::cmd("SET").arg(key).arg(raw).query::<()>(&mut conn)?;
    Ok(())
}

pub async fn delete_state(gid: GuildId, cid: ChannelId) -> Result<(), RedisError> {
    let mut conn = crate::redis::get_conn().await?;
    let key = state_key(gid, cid);
    redis::cmd("DEL").arg(key).query::<()>(&mut conn)?;
    Ok(())
}

pub fn position_after_dealer(state: &PokerGameState, offset: usize) -> Option<UserId> {
    if state.players.is_empty() {
        return None;
    }
    let idx = (state.dealer_index + offset) % state.players.len();
    Some(UserId::from(state.players[idx]))
}

pub fn next_active_after_dealer(state: &PokerGameState) -> Option<UserId> {
    let active = state.active_players();
    let dealer = UserId::from(state.players[state.dealer_index]);
    if let Some(pos) = active.iter().position(|&u| u == dealer) {
        return active.get((pos + 1) % active.len()).copied();
    }
    active.first().copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn betting_round_not_complete_after_blinds() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.advance_phase();
        state.place_bet(UserId::from(2), 25); // small blind
        state.place_bet(UserId::from(1), 50); // big blind
        state.current_bet = 50;
        assert!(!state.betting_round_complete());
    }

    #[test]
    fn betting_round_complete_after_both_call() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.advance_phase();
        state.place_bet(UserId::from(2), 50);
        state.place_bet(UserId::from(1), 50);
        state.current_bet = 50;
        state.mark_acted(UserId::from(2));
        state.mark_acted(UserId::from(1));
        assert!(state.betting_round_complete());
    }

    #[test]
    fn advance_turn_moves_to_next_unacted_player() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.current_player_index = 1;
        state.advance_turn();
        assert_eq!(state.current_player(), Some(UserId::from(1)));
    }
}
