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
    pub all_in: HashSet<u64>,
    pub acted_this_round: HashSet<u64>,
    pub community_cards: Vec<u8>,
    pub hole_cards: HashMap<u64, Vec<u8>>,
    pub lobby_message_id: Option<u64>,
    pub status_message_id: Option<u64>,
    pub action_prompt_message_id: Option<u64>,
    pub turn_seconds: u64,
    pub lobby_seconds: u64,
    pub has_bot: bool,
    pub bot_balance: u64,
    pub small_blind: u64,
    pub big_blind: u64,
    pub turn_timer_id: u64,
    pub max_hand_bet: Option<u64>,
    pub hand_bets: HashMap<u64, u64>,
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
            all_in: HashSet::new(),
            acted_this_round: HashSet::new(),
            community_cards: Vec::new(),
            hole_cards: HashMap::new(),
            lobby_message_id: None,
            status_message_id: None,
            action_prompt_message_id: None,
            turn_seconds: DEFAULT_TURN_SECONDS,
            lobby_seconds: DEFAULT_LOBBY_SECONDS,
            has_bot: false,
            bot_balance: DEFAULT_BOT_BALANCE,
            small_blind: 25,
            big_blind: 50,
            turn_timer_id: 0,
            max_hand_bet: None,
            hand_bets: HashMap::new(),
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

    pub fn is_all_in(&self, uid: UserId) -> bool {
        self.all_in.contains(&uid.0)
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
        let current_hand = self.hand_bet(uid);
        self.hand_bets.insert(uid.0, current_hand + amount);
    }

    pub fn hand_bet(&self, uid: UserId) -> u64 {
        *self.hand_bets.get(&uid.0).unwrap_or(&0)
    }

    pub fn would_exceed_max_hand_bet(&self, uid: UserId, amount: u64) -> bool {
        match self.max_hand_bet {
            Some(max) => self.hand_bet(uid).saturating_add(amount) > max,
            None => false,
        }
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
                if self.is_folded(next) || self.is_all_in(next) {
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
            if self.is_all_in(uid) {
                continue;
            }
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
                if !self.is_folded(current) && !self.is_all_in(current) {
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

/// Load the game state, or a GenericError if there is no active game.
pub async fn load_state_or_err(
    gid: GuildId,
    cid: ChannelId,
) -> Result<PokerGameState, crate::errors::GenericError> {
    load_state(gid, cid)
        .await
        .map_err(|e| crate::errors::GenericError::new(&e.to_string()))?
        .ok_or(crate::errors::GenericError::new(&"No poker game found."))
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

/// Save the game state, mapping any redis error to a GenericError.
pub async fn save_state_or_err(
    gid: GuildId,
    cid: ChannelId,
    state: &PokerGameState,
) -> Result<(), crate::errors::GenericError> {
    save_state(gid, cid, state)
        .await
        .map_err(|e| crate::errors::GenericError::new(&e.to_string()))
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

    #[test]
    fn all_in_player_skipped_in_turn_order() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.add_player(UserId::from(3));
        state.current_player_index = 0;
        state.all_in.insert(1);
        state.acted_this_round.insert(2);
        state.place_bet(UserId::from(2), 50);
        state.current_bet = 50;
        assert!(state.advance_turn());
        assert_eq!(state.current_player(), Some(UserId::from(3)));
    }

    #[test]
    fn betting_round_complete_with_all_in_short_stack() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.add_player(UserId::from(3));
        state.current_bet = 100;
        state.place_bet(UserId::from(1), 20); // all-in short stack
        state.all_in.insert(1);
        state.place_bet(UserId::from(2), 100);
        state.acted_this_round.insert(2);
        state.place_bet(UserId::from(3), 100);
        state.acted_this_round.insert(3);
        assert!(state.betting_round_complete());
    }

    #[test]
    fn new_game_has_sensible_defaults() {
        let state = PokerGameState::new(UserId::from(1));
        assert_eq!(state.phase, Phase::Lobby);
        assert_eq!(state.players, vec![1]);
        assert_eq!(state.pot, 0);
        assert_eq!(state.current_bet, 0);
        assert_eq!(state.small_blind, 25);
        assert_eq!(state.big_blind, 50);
        assert!(!state.has_bot);
    }

    #[test]
    fn add_player_rejects_duplicates() {
        let mut state = PokerGameState::new(UserId::from(1));
        assert!(!state.add_player(UserId::from(1))); // host already in
        assert!(state.add_player(UserId::from(2)));
        assert!(!state.add_player(UserId::from(2)));
        assert_eq!(state.players.len(), 2);
    }

    #[test]
    fn add_bot_only_once() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_bot();
        state.add_bot();
        assert!(state.has_bot);
        assert_eq!(
            state.players.iter().filter(|&&p| p == BOT_USER_ID).count(),
            1
        );
        assert!(is_bot(UserId::from(BOT_USER_ID)));
        assert!(!is_bot(UserId::from(1)));
    }

    #[test]
    fn remove_player_marks_folded_and_drops_state() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.place_bet(UserId::from(2), 50);
        state.set_hole_cards(UserId::from(2), vec![0, 1]);

        assert!(state.remove_player(UserId::from(2)));
        assert!(!state.players.contains(&2));
        assert!(state.is_folded(UserId::from(2)));
        assert_eq!(state.player_bet(UserId::from(2)), 0);
        assert!(state.hole_cards_eval(UserId::from(2)).is_empty());
        // removing a player who isn't there is a no-op
        assert!(!state.remove_player(UserId::from(2)));
    }

    #[test]
    fn remove_player_fixes_out_of_range_turn_index() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.current_player_index = 1; // pointing at player 2
        state.remove_player(UserId::from(2));
        assert_eq!(state.current_player_index, 0);
        assert_eq!(state.current_player(), Some(UserId::from(1)));
    }

    #[test]
    fn place_bet_accumulates_pot_and_hand_bets() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.place_bet(UserId::from(1), 25);
        state.place_bet(UserId::from(1), 50);
        assert_eq!(state.pot, 75);
        assert_eq!(state.player_bet(UserId::from(1)), 75);
        assert_eq!(state.hand_bet(UserId::from(1)), 75);
    }

    #[test]
    fn hand_bet_survives_round_bet_reset() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.place_bet(UserId::from(1), 50);
        state.advance_phase(); // -> PreFlop, clears round bets
        state.advance_phase(); // -> Flop, clears round bets
        assert_eq!(state.player_bet(UserId::from(1)), 0);
        assert_eq!(state.hand_bet(UserId::from(1)), 50);
    }

    #[test]
    fn max_hand_bet_enforced() {
        let mut state = PokerGameState::new(UserId::from(1));
        // no cap by default
        assert!(!state.would_exceed_max_hand_bet(UserId::from(1), 1_000_000));

        state.max_hand_bet = Some(100);
        state.place_bet(UserId::from(1), 60);
        assert!(!state.would_exceed_max_hand_bet(UserId::from(1), 40));
        assert!(state.would_exceed_max_hand_bet(UserId::from(1), 41));
    }

    #[test]
    fn fold_marks_player_acted_and_folded() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.fold(UserId::from(1));
        assert!(state.is_folded(UserId::from(1)));
        assert!(state.acted_this_round.contains(&1));
        assert!(state.active_players().is_empty());
    }

    #[test]
    fn clear_acted_except_keeps_only_raiser() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.add_player(UserId::from(3));
        state.mark_acted(UserId::from(1));
        state.mark_acted(UserId::from(2));
        state.clear_acted_except(UserId::from(3));
        assert!(!state.acted_this_round.contains(&1));
        assert!(!state.acted_this_round.contains(&2));
        assert!(state.acted_this_round.contains(&3));
    }

    #[test]
    fn advance_phase_cycles_and_resets_round_state() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));

        state.advance_phase(); // Lobby -> PreFlop
        assert_eq!(state.phase, Phase::PreFlop);
        assert_eq!(state.current_bet, state.big_blind);

        state.place_bet(UserId::from(1), 50);
        state.mark_acted(UserId::from(1));

        state.advance_phase(); // PreFlop -> Flop
        assert_eq!(state.phase, Phase::Flop);
        assert_eq!(state.current_bet, 0);
        assert!(state.round_bets.is_empty());
        assert!(state.acted_this_round.is_empty());

        state.advance_phase(); // Flop -> Turn
        assert_eq!(state.phase, Phase::Turn);
        state.advance_phase(); // Turn -> River
        assert_eq!(state.phase, Phase::River);
        state.advance_phase(); // River -> Showdown
        assert_eq!(state.phase, Phase::Showdown);
        // showdown is terminal
        state.advance_phase();
        assert_eq!(state.phase, Phase::Showdown);
    }

    #[test]
    fn skip_folded_players_moves_past_folded() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.add_player(UserId::from(3));
        state.current_player_index = 0;
        state.fold(UserId::from(1));
        state.skip_folded_players();
        assert_eq!(state.current_player(), Some(UserId::from(2)));
    }

    #[test]
    fn skip_folded_players_all_folded_does_not_hang() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.fold(UserId::from(1));
        state.fold(UserId::from(2));
        state.skip_folded_players(); // must terminate
    }

    #[test]
    fn betting_round_complete_single_active_player() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.fold(UserId::from(2));
        assert!(state.betting_round_complete());
        assert!(!state.advance_turn());
    }

    #[test]
    fn position_after_dealer_offsets() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.add_player(UserId::from(3));
        state.dealer_index = 0;
        assert_eq!(position_after_dealer(&state, 1), Some(UserId::from(2)));
        assert_eq!(position_after_dealer(&state, 2), Some(UserId::from(3)));
        assert_eq!(position_after_dealer(&state, 3), Some(UserId::from(1)));
    }

    #[test]
    fn position_after_dealer_empty_table() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.players.clear();
        assert_eq!(position_after_dealer(&state, 1), None);
    }

    #[test]
    fn next_active_after_dealer_skips_folded() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.add_player(UserId::from(3));
        state.dealer_index = 0;
        state.fold(UserId::from(2));
        assert_eq!(next_active_after_dealer(&state), Some(UserId::from(3)));
    }

    #[test]
    fn hole_and_community_cards_convert_to_eval() {
        let mut state = PokerGameState::new(UserId::from(1));
        // int 0 = ONE of DIAMONDS -> ace high, int 12 = KING of DIAMONDS
        state.set_hole_cards(UserId::from(1), vec![0, 12]);
        state.add_community_card(13); // ONE of HEARTS

        let hole = state.hole_cards_eval(UserId::from(1));
        assert_eq!(hole, vec![EvalCard::new(14, 0), EvalCard::new(13, 0)]);

        let community = state.community_cards_eval();
        assert_eq!(community, vec![EvalCard::new(14, 1)]);

        let all = state.all_cards_for_player(UserId::from(1));
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn state_key_format() {
        assert_eq!(
            state_key(GuildId::from(111), ChannelId::from(222)),
            "poker_state_111_222"
        );
    }
}
