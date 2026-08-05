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
        Some(self.cmp(other))
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

    let is_flush = cards
        .iter()
        .map(|c| c.suit)
        .collect::<std::collections::HashSet<_>>()
        .len()
        == 1;
    let straight_high = check_straight(&ranks);

    if let (true, Some(high)) = (is_flush, straight_high) {
        return vec![8, high];
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

/// Format cards as Discord custom emoji (two-line colored card faces).
pub fn cards_to_discord_emojis(cards: &[EvalCard]) -> String {
    use crate::utils::card_ascii::{BLACK_CARDS, RED_CARDS, SUITES};

    let mut values = String::new();
    let mut suits = String::new();
    for card in cards {
        let idx = match card.rank {
            14 => 0,
            2..=13 => (card.rank - 1) as usize,
            _ => 0,
        };
        let value = match card.suit {
            0 | 1 => RED_CARDS[idx],
            _ => BLACK_CARDS[idx],
        };
        values.push_str(value);
        values.push(' ');
        suits.push_str(SUITES[card.suit as usize]);
        suits.push(' ');
    }
    format!("{}\n{}", values, suits)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(rank: Rank, suit: Suit) -> EvalCard {
        EvalCard::new(rank, suit)
    }

    /// Build a 7-card hand from 5 "core" cards plus two junk cards that
    /// don't interact (different suits, off-straight ranks).
    fn seven(core: [EvalCard; 5], junk: [EvalCard; 2]) -> Vec<EvalCard> {
        core.iter().chain(junk.iter()).copied().collect()
    }

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

    #[test]
    fn test_discord_emojis_format() {
        let cards = vec![
            EvalCard::new(14, 0), // Ace of diamonds (red)
            EvalCard::new(13, 3), // King of spades (black)
        ];
        let emoji = cards_to_discord_emojis(&cards);
        assert!(emoji.contains("<:rA:"));
        assert!(emoji.contains("<:s_diamonds:"));
        assert!(emoji.contains("<:bK:"));
        assert!(emoji.contains("<:s_spades:"));
        assert!(emoji.contains('\n'));
    }

    // ---- category detection ----

    #[test]
    fn detects_straight_flush() {
        let hand = seven(
            [c(9, 2), c(8, 2), c(7, 2), c(6, 2), c(5, 2)],
            [c(14, 0), c(13, 1)],
        );
        assert_eq!(evaluate_seven(&hand)[0], 8);
    }

    #[test]
    fn detects_four_of_a_kind_with_kicker() {
        let hand = seven(
            [c(9, 0), c(9, 1), c(9, 2), c(9, 3), c(14, 0)],
            [c(2, 1), c(3, 2)],
        );
        assert_eq!(evaluate_seven(&hand), vec![7, 9, 14]);
    }

    #[test]
    fn detects_full_house() {
        let hand = seven(
            [c(10, 0), c(10, 1), c(10, 2), c(4, 0), c(4, 1)],
            [c(7, 3), c(13, 0)],
        );
        assert_eq!(evaluate_seven(&hand), vec![6, 10, 4]);
    }

    #[test]
    fn detects_flush() {
        let hand = seven(
            [c(14, 1), c(11, 1), c(9, 1), c(6, 1), c(2, 1)],
            [c(3, 0), c(4, 2)],
        );
        let rank = evaluate_seven(&hand);
        assert_eq!(rank, vec![5, 14, 11, 9, 6, 2]);
    }

    #[test]
    fn detects_straight() {
        let hand = seven(
            [c(10, 0), c(9, 1), c(8, 2), c(7, 0), c(6, 1)],
            [c(2, 2), c(14, 3)],
        );
        assert_eq!(evaluate_seven(&hand), vec![4, 10]);
    }

    #[test]
    fn detects_wheel_straight_ace_low() {
        let hand = seven(
            [c(14, 0), c(2, 1), c(3, 2), c(4, 0), c(5, 1)],
            [c(9, 2), c(11, 3)],
        );
        assert_eq!(evaluate_seven(&hand), vec![4, 5]);
    }

    #[test]
    fn detects_three_of_a_kind_with_kickers() {
        let hand = seven(
            [c(8, 0), c(8, 1), c(8, 2), c(14, 0), c(11, 1)],
            [c(2, 2), c(4, 3)],
        );
        assert_eq!(evaluate_seven(&hand), vec![3, 8, 14, 11]);
    }

    #[test]
    fn detects_two_pair_with_kicker() {
        let hand = seven(
            [c(12, 0), c(12, 1), c(5, 2), c(5, 0), c(9, 1)],
            [c(2, 2), c(3, 3)],
        );
        assert_eq!(evaluate_seven(&hand), vec![2, 12, 5, 9]);
    }

    #[test]
    fn detects_pair_with_kickers() {
        let hand = seven(
            [c(11, 0), c(11, 1), c(14, 2), c(9, 0), c(4, 1)],
            [c(2, 2), c(6, 3)],
        );
        assert_eq!(evaluate_seven(&hand), vec![1, 11, 14, 9, 6]);
    }

    #[test]
    fn detects_high_card() {
        let hand = seven(
            [c(14, 0), c(12, 1), c(9, 2), c(6, 0), c(3, 1)],
            [c(2, 2), c(4, 3)],
        );
        assert_eq!(evaluate_seven(&hand), vec![0, 14, 12, 9, 6, 4]);
    }

    // ---- category ordering ----

    #[test]
    fn category_ordering_holds() {
        let straight_flush = seven(
            [c(9, 2), c(8, 2), c(7, 2), c(6, 2), c(5, 2)],
            [c(2, 0), c(3, 1)],
        );
        let quads = seven(
            [c(9, 0), c(9, 1), c(9, 2), c(9, 3), c(2, 0)],
            [c(3, 1), c(4, 2)],
        );
        let full_house = seven(
            [c(9, 0), c(9, 1), c(9, 2), c(2, 0), c(2, 1)],
            [c(3, 2), c(4, 3)],
        );
        let flush = seven(
            [c(14, 1), c(11, 1), c(9, 1), c(6, 1), c(2, 1)],
            [c(3, 0), c(4, 2)],
        );
        let straight = seven(
            [c(10, 0), c(9, 1), c(8, 2), c(7, 0), c(6, 1)],
            [c(2, 2), c(14, 3)],
        );
        let trips = seven(
            [c(8, 0), c(8, 1), c(8, 2), c(14, 0), c(11, 1)],
            [c(2, 2), c(4, 3)],
        );
        let two_pair = seven(
            [c(12, 0), c(12, 1), c(5, 2), c(5, 0), c(9, 1)],
            [c(2, 2), c(3, 3)],
        );
        let pair = seven(
            [c(11, 0), c(11, 1), c(14, 2), c(9, 0), c(4, 1)],
            [c(2, 2), c(6, 3)],
        );
        let high = seven(
            [c(14, 0), c(12, 1), c(9, 2), c(6, 0), c(3, 1)],
            [c(2, 2), c(4, 3)],
        );

        let ordered = [
            &straight_flush,
            &quads,
            &full_house,
            &flush,
            &straight,
            &trips,
            &two_pair,
            &pair,
            &high,
        ];
        for w in ordered.windows(2) {
            assert!(
                evaluate_seven(w[0]) > evaluate_seven(w[1]),
                "category ordering violated"
            );
        }
    }

    // ---- tie breaking ----

    #[test]
    fn higher_pair_wins() {
        let aces = seven(
            [c(14, 0), c(14, 1), c(9, 2), c(6, 0), c(3, 1)],
            [c(2, 2), c(4, 3)],
        );
        let kings = seven(
            [c(13, 0), c(13, 1), c(9, 2), c(6, 0), c(3, 1)],
            [c(2, 2), c(4, 3)],
        );
        assert!(evaluate_seven(&aces) > evaluate_seven(&kings));
    }

    #[test]
    fn same_pair_kicker_decides() {
        let ace_kicker = seven(
            [c(11, 0), c(11, 1), c(14, 2), c(9, 0), c(4, 1)],
            [c(2, 2), c(6, 3)],
        );
        let king_kicker = seven(
            [c(11, 2), c(11, 3), c(13, 0), c(9, 1), c(4, 2)],
            [c(2, 0), c(6, 1)],
        );
        assert!(evaluate_seven(&ace_kicker) > evaluate_seven(&king_kicker));
    }

    #[test]
    fn two_pair_higher_top_pair_wins() {
        let aces_up = seven(
            [c(14, 0), c(14, 1), c(2, 2), c(2, 0), c(9, 1)],
            [c(3, 2), c(4, 3)],
        );
        let kings_up = seven(
            [c(13, 0), c(13, 1), c(12, 2), c(12, 0), c(9, 1)],
            [c(3, 2), c(4, 3)],
        );
        assert!(evaluate_seven(&aces_up) > evaluate_seven(&kings_up));
    }

    #[test]
    fn identical_hands_tie() {
        let a = seven(
            [c(10, 0), c(10, 1), c(14, 2), c(9, 0), c(4, 1)],
            [c(2, 2), c(6, 3)],
        );
        let b = seven(
            [c(10, 2), c(10, 3), c(14, 0), c(9, 1), c(4, 2)],
            [c(2, 0), c(6, 1)],
        );
        assert_eq!(evaluate_seven(&a), evaluate_seven(&b));
    }

    #[test]
    fn straight_higher_top_card_wins() {
        let broadway = seven(
            [c(14, 0), c(13, 1), c(12, 2), c(11, 0), c(10, 1)],
            [c(2, 2), c(3, 3)],
        );
        let wheel = seven(
            [c(14, 1), c(2, 0), c(3, 1), c(4, 2), c(5, 0)],
            [c(9, 2), c(11, 3)],
        );
        assert!(evaluate_seven(&broadway) > evaluate_seven(&wheel));
    }

    // ---- best 5 of 7 ----

    #[test]
    fn picks_best_five_of_seven() {
        // two pair on board + pocket aces -> two pair, aces up
        let hand = vec![
            c(14, 0),
            c(14, 1), // pocket aces
            c(13, 2),
            c(13, 0), // board pair of kings
            c(2, 1),
            c(7, 2),
            c(9, 3),
        ];
        assert_eq!(evaluate_seven(&hand), vec![2, 14, 13, 9]);
    }

    #[test]
    fn three_of_a_kind_on_board_plus_pair_is_full_house() {
        let hand = vec![
            c(8, 0),
            c(8, 1),
            c(8, 2), // trips on board
            c(5, 0),
            c(5, 1), // pocket pair
            c(11, 2),
            c(3, 3),
        ];
        assert_eq!(evaluate_seven(&hand), vec![6, 8, 5]);
    }

    #[test]
    fn four_flush_cards_do_not_make_a_flush() {
        let hand = vec![
            c(14, 1),
            c(11, 1),
            c(9, 1),
            c(6, 1), // only four hearts
            c(2, 0),
            c(3, 2),
            c(4, 3),
        ];
        assert_eq!(evaluate_seven(&hand)[0], 0); // high card
    }

    // ---- helpers ----

    #[test]
    fn rank_to_string_covers_all_categories() {
        let cases = [
            (8, "Straight Flush"),
            (7, "Four of a Kind"),
            (6, "Full House"),
            (5, "Flush"),
            (4, "Straight"),
            (3, "Three of a Kind"),
            (2, "Two Pair"),
            (1, "Pair"),
            (0, "High Card"),
        ];
        for (cat, name) in cases {
            assert_eq!(rank_to_string(&[cat]), name);
        }
        assert_eq!(rank_to_string(&[]), "Unknown");
    }

    #[test]
    fn card_to_emoji_formats_ranks_and_suits() {
        assert_eq!(card_to_emoji(c(14, 0)), "A♦");
        assert_eq!(card_to_emoji(c(13, 1)), "K♥");
        assert_eq!(card_to_emoji(c(10, 2)), "10♣");
        assert_eq!(card_to_emoji(c(2, 3)), "2♠");
    }

    #[test]
    fn card_tuple_to_eval_maps_ace_high() {
        use crate::sql::structs::{Card, Suite};
        assert_eq!(card_tuple_to_eval((Card::ONE, Suite::SPADES)), c(14, 3));
        assert_eq!(card_tuple_to_eval((Card::KING, Suite::HEARTS)), c(13, 1));
        assert_eq!(card_tuple_to_eval((Card::TWO, Suite::DIAMONDS)), c(2, 0));
    }
}
