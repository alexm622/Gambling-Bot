// Game session / state machine for 5-card draw poker.

use std::collections::{HashMap, HashSet};

use redis::RedisError;
use serde::{Deserialize, Serialize};
use serenity::model::prelude::{ChannelId, GuildId, UserId};

use crate::redis::poker as poker_redis;

const DEFAULT_TURN_SECONDS: u64 = 30;
const DEFAULT_LOBBY_SECONDS: u64 = 60;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Lobby,
    FirstBet,
    Draw,
    SecondBet,
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
    pub lobby_message_id: Option<u64>,
    pub status_message_id: Option<u64>,
    pub turn_seconds: u64,
    pub lobby_seconds: u64,
}

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
            lobby_message_id: None,
            status_message_id: None,
            turn_seconds: DEFAULT_TURN_SECONDS,
            lobby_seconds: DEFAULT_LOBBY_SECONDS,
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

    pub fn remove_player(&mut self, uid: UserId) -> bool {
        if let Some(pos) = self.players.iter().position(|&id| id == uid.0) {
            self.players.remove(pos);
            self.folded.insert(uid.0);
            self.round_bets.remove(&uid.0);
            self.acted_this_round.insert(uid.0);
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

    /// Advance to the next active player. Returns false if the betting round is complete.
    pub fn advance_turn(&mut self) -> bool {
        let active = self.active_players();
        if active.len() < 2 {
            return false;
        }

        for _ in 0..active.len() {
            self.current_player_index = (self.current_player_index + 1) % self.players.len();
            if let Some(next) = self.current_player() {
                if !self.is_folded(next) && !self.acted_this_round.contains(&next.0) {
                    return true;
                }
                // if already acted, check if they still need to match a raise
                if !self.is_folded(next) {
                    let player_bet = self.player_bet(next);
                    if player_bet < self.current_bet {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if betting round is over: every active player has matched the current bet.
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
                self.phase = Phase::FirstBet;
                self.current_player_index = 0;
                self.current_bet = 0;
                self.round_bets.clear();
                self.acted_this_round.clear();
            }
            Phase::FirstBet => {
                self.phase = Phase::Draw;
                self.current_player_index = 0;
                self.current_bet = 0;
                self.round_bets.clear();
                self.acted_this_round.clear();
            }
            Phase::Draw => {
                self.phase = Phase::SecondBet;
                self.current_player_index = 0;
                self.current_bet = 0;
                self.round_bets.clear();
                self.acted_this_round.clear();
            }
            Phase::SecondBet => {
                self.phase = Phase::Showdown;
                self.current_player_index = 0;
            }
            _ => {}
        }
    }

    pub fn skip_folded_players(&mut self) {
        while let Some(current) = self.current_player() {
            if !self.is_folded(current) {
                break;
            }
            self.current_player_index = (self.current_player_index + 1) % self.players.len();
        }
    }
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

pub async fn deal_hands(
    gid: GuildId,
    cid: ChannelId,
    players: &[UserId],
) -> Result<(), RedisError> {
    for uid in players {
        let _ = poker_redis::get_user_hand(gid, cid, *uid).await?;
    }
    Ok(())
}
