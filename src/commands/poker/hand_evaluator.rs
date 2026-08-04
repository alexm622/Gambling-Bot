// Simple Texas Hold'em 7-card hand evaluator.

use std::cmp::Ordering;

use crate::sql::structs::{Card, Suite};

pub type Rank = u8;
pub type Suit = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvalCard {
    pub rank: Rank,
    pub suit: Suit,
}

impl EvalCard {
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
    }
}

impl PartialOrd for EvalCard {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.rank.cmp(&other.rank))
    }
}

impl Ord for EvalCard {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank.cmp(&other.rank)
    }
}

pub fn card_tuple_to_eval(card: (Card, Suite)) -> EvalCard {
    let rank = match card.0 {
        Card::ONE => 14, // Ace high by default
        Card::TWO => 2,
        Card::THREE => 3,
        Card::FOUR => 4,
        Card::FIVE => 5,
        Card::SIX => 6,
        Card::SEVEN => 7,
        Card::EIGHT => 8,
        Card::NINE => 9,
        Card::TEN => 10,
        Card::JACK => 11,
        Card::QUEEN => 12,
        Card::KING => 13,
    };
    EvalCard::new(rank, card.1 as u8)
}

/// Evaluate a 7-card hand and return a rank vector. Higher is better.
pub fn evaluate_seven(cards: &[EvalCard]) -> Vec<u8> {
    let mut best: Vec<u8> = vec![0];

    // check all 21 combinations of 5 cards
    let n = cards.len();
    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                for l in (k + 1)..n {
                    for m in (l + 1)..n {
                        let five = vec![cards[i], cards[j], cards[k], cards[l], cards[m]];
                        let rank = evaluate_five(&five);
                        if rank > best {
                            best = rank;
                        }
                    }
                }
            }
        }
    }

    best
}

fn evaluate_five(cards: &[EvalCard]) -> Vec<u8> {
    let mut ranks: Vec<u8> = cards.iter().map(|c| c.rank).collect();
    ranks.sort_unstable_by(|a, b| b.cmp(a)); // descending

    let mut counts: std::collections::HashMap<u8, u8> = std::collections::HashMap::new();
    for r in &ranks {
        *counts.entry(*r).or_insert(0) += 1;
    }

    let mut count_groups: Vec<(u8, u8)> = counts.into_iter().collect();
    // sort by count desc, then rank desc
    count_groups.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));

    let is_flush = cards.iter().map(|c| c.suit).collect::<std::collections::HashSet<_>>().len() == 1;
    let straight_high = check_straight(&ranks);

    if is_flush && straight_high.is_some() {
        return vec![8, straight_high.unwrap()];
    }

    // four of a kind
    if count_groups[0].1 == 4 {
        let quad = count_groups[0].0;
        let kicker = count_groups[1].0;
        return vec![7, quad, kicker];
    }

    // full house
    if count_groups[0].1 == 3 && count_groups[1].1 >= 2 {
        return vec![6, count_groups[0].0, count_groups[1].0];
    }

    // flush
    if is_flush {
        let mut result = vec![5];
        result.extend(ranks.iter());
        return result;
    }

    // straight
    if let Some(high) = straight_high {
        return vec![4, high];
    }

    // three of a kind
    if count_groups[0].1 == 3 {
        let mut result = vec![3, count_groups[0].0];
        for (rank, count) in count_groups.iter().skip(1) {
            for _ in 0..*count {
                result.push(*rank);
            }
        }
        return result;
    }

    // two pair
    if count_groups[0].1 == 2 && count_groups[1].1 == 2 {
        let high_pair = count_groups[0].0.max(count_groups[1].0);
        let low_pair = count_groups[0].0.min(count_groups[1].0);
        let kicker = count_groups[2].0;
        return vec![2, high_pair, low_pair, kicker];
    }

    // pair
    if count_groups[0].1 == 2 {
        let mut result = vec![1, count_groups[0].0];
        for (rank, count) in count_groups.iter().skip(1) {
            for _ in 0..*count {
                result.push(*rank);
            }
        }
        return result;
    }

    // high card
    let mut result = vec![0];
    result.extend(ranks.iter());
    result
}

fn check_straight(sorted_ranks_desc: &[u8]) -> Option<u8> {
    let mut unique: Vec<u8> = sorted_ranks_desc.to_vec();
    unique.dedup();

    if unique.len() < 5 {
        return None;
    }

    // check normal straights
    for window in unique.windows(5) {
        if window[0] - window[4] == 4 {
            return Some(window[0]);
        }
    }

    // check low straight A-2-3-4-5
    if unique.contains(&14)
        && unique.contains(&2)
        && unique.contains(&3)
        && unique.contains(&4)
        && unique.contains(&5)
    {
        return Some(5);
    }

    None
}

pub fn rank_to_string(rank: &[u8]) -> String {
    if rank.is_empty() {
        return "Unknown".to_string();
    }

    let category = match rank[0] {
        8 => "Straight Flush",
        7 => "Four of a Kind",
        6 => "Full House",
        5 => "Flush",
        4 => "Straight",
        3 => "Three of a Kind",
        2 => "Two Pair",
        1 => "Pair",
        _ => "High Card",
    };

    category.to_string()
}

pub fn card_to_emoji(card: EvalCard) -> String {
    let rank = match card.rank {
        14 => "A",
        13 => "K",
        12 => "Q",
        11 => "J",
        10 => "10",
        9 => "9",
        8 => "8",
        7 => "7",
        6 => "6",
        5 => "5",
        4 => "4",
        3 => "3",
        2 => "2",
        _ => "?",
    };
    let suit = match card.suit {
        0 => "♦",
        1 => "♥",
        2 => "♣",
        3 => "♠",
        _ => "?",
    };
    format!("{}{}", rank, suit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_royal_flush_beats_straight_flush() {
        let royal = vec![
            EvalCard::new(14, 0),
            EvalCard::new(13, 0),
            EvalCard::new(12, 0),
            EvalCard::new(11, 0),
            EvalCard::new(10, 0),
            EvalCard::new(2, 1),
            EvalCard::new(3, 2),
        ];
        let straight_flush = vec![
            EvalCard::new(9, 1),
            EvalCard::new(8, 1),
            EvalCard::new(7, 1),
            EvalCard::new(6, 1),
            EvalCard::new(5, 1),
            EvalCard::new(2, 0),
            EvalCard::new(3, 0),
        ];
        assert!(evaluate_seven(&royal) > evaluate_seven(&straight_flush));
    }

    #[test]
    fn test_pair_beats_high_card() {
        let pair = vec![
            EvalCard::new(14, 0),
            EvalCard::new(14, 1),
            EvalCard::new(2, 0),
            EvalCard::new(3, 1),
            EvalCard::new(5, 2),
            EvalCard::new(7, 3),
            EvalCard::new(9, 0),
        ];
        let high = vec![
            EvalCard::new(14, 0),
            EvalCard::new(13, 1),
            EvalCard::new(11, 0),
            EvalCard::new(9, 1),
            EvalCard::new(7, 2),
            EvalCard::new(5, 3),
            EvalCard::new(3, 0),
        ];
        assert!(evaluate_seven(&pair) > evaluate_seven(&high));
    }
}
