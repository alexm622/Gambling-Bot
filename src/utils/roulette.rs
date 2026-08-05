//roulette

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::sql::structs::BetResult;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct SpinResult {
    pub value: u8,
    pub color: Color,
    pub oddness: bool,
}

impl fmt::Display for SpinResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Value: {}, Color: {}, Oddness: {}",
            self.value, self.color, self.oddness
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
    RED,
    BLACK,
    GREEN,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Color::RED => write!(f, "Red"),
            Color::BLACK => write!(f, "Black"),
            Color::GREEN => write!(f, "Green"),
        }
    }
}

//spin the table
pub fn get_spin() -> SpinResult {
    let rng = rand::random::<u8>() % 38;

    SpinResult {
        value: rng,
        color: get_color(rng),
        oddness: rng.is_multiple_of(2),
    }
}

//get the color that was landed on
pub fn get_color(rng: u8) -> Color {
    if rng == 0 || rng == 37 {
        return Color::GREEN;
    }

    if rng <= 10 || (19..=28).contains(&rng) {
        if rng.is_multiple_of(2) {
            return Color::BLACK;
        } else {
            return Color::RED;
        }
    }

    //everything else
    if rng.is_multiple_of(2) {
        Color::RED
    } else {
        Color::BLACK
    }
}

//check all the current bets against the table
//the stake was already deducted when the bet was placed, so `net` starts at
//the stake: on a win it becomes stake + (stake * payout), on a loss it flips
//negative so it can be reported back to the player
pub fn bet_check(bet: &mut BetResult, spin: SpinResult) {
    let green = spin.color == Color::GREEN;

    // even-money bets all lose on 0/00, as in real roulette
    let won = match bet.bet_type {
        0 => spin.color == Color::RED,
        1 => spin.color == Color::BLACK,
        2 => !green && spin.oddness,     // even
        3 => !green && !spin.oddness,    // odd
        4 => !green && spin.value <= 18, // low (1-18)
        5 => !green && spin.value >= 19, // high (19-36)
        6 => bet.specific_bet == Some(spin.value),
        7 => green, // green covers 0 and 00
        _ => false,
    };

    if won {
        let payout: i64 = match bet.bet_type {
            6 => 35, // single number
            7 => 17, // green covers 2 of 38 numbers
            _ => 1,  // even-money bets
        };
        bet.net += bet.net * payout;
    } else {
        bet.net *= -1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // standard American roulette wheel (37 represents 00)
    const REDS: [u8; 18] = [
        1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36,
    ];
    const BLACKS: [u8; 18] = [
        2, 4, 6, 8, 10, 11, 13, 15, 17, 20, 22, 24, 26, 28, 29, 31, 33, 35,
    ];

    fn spin(value: u8) -> SpinResult {
        SpinResult {
            value,
            color: get_color(value),
            oddness: value % 2 == 0,
        }
    }

    fn bet(amount: i64, bet_type: u8, specific_bet: Option<u8>) -> BetResult {
        BetResult {
            user_id: 1,
            net: amount,
            bet_type,
            specific_bet,
        }
    }

    #[test]
    fn wheel_colors_match_standard_layout() {
        for &n in REDS.iter() {
            assert_eq!(get_color(n), Color::RED, "{} should be red", n);
        }
        for &n in BLACKS.iter() {
            assert_eq!(get_color(n), Color::BLACK, "{} should be black", n);
        }
        assert_eq!(get_color(0), Color::GREEN);
        assert_eq!(get_color(37), Color::GREEN); // 00
    }

    #[test]
    fn every_number_has_a_color() {
        for n in 0..=37u8 {
            let _ = get_color(n); // must not panic
        }
    }

    #[test]
    fn spin_produces_valid_results() {
        for _ in 0..1000 {
            let s = get_spin();
            assert!(s.value < 38);
            assert_eq!(s.color, get_color(s.value));
            assert_eq!(s.oddness, s.value % 2 == 0);
        }
    }

    #[test]
    fn red_bet_wins_on_red_loses_on_black() {
        let mut b = bet(100, 0, None);
        bet_check(&mut b, spin(1)); // red
        assert_eq!(b.net, 200);

        let mut b = bet(100, 0, None);
        bet_check(&mut b, spin(2)); // black
        assert_eq!(b.net, -100);

        let mut b = bet(100, 0, None);
        bet_check(&mut b, spin(0)); // green
        assert_eq!(b.net, -100);
    }

    #[test]
    fn black_bet_wins_on_black_loses_on_red() {
        let mut b = bet(100, 1, None);
        bet_check(&mut b, spin(2)); // black
        assert_eq!(b.net, 200);

        let mut b = bet(100, 1, None);
        bet_check(&mut b, spin(1)); // red
        assert_eq!(b.net, -100);
    }

    #[test]
    fn even_and_odd_bets() {
        let mut b = bet(100, 2, None); // even
        bet_check(&mut b, spin(4));
        assert_eq!(b.net, 200);

        let mut b = bet(100, 2, None);
        bet_check(&mut b, spin(5));
        assert_eq!(b.net, -100);

        let mut b = bet(100, 3, None); // odd
        bet_check(&mut b, spin(5));
        assert_eq!(b.net, 200);

        let mut b = bet(100, 3, None);
        bet_check(&mut b, spin(4));
        assert_eq!(b.net, -100);
    }

    #[test]
    fn low_and_high_bets() {
        let mut b = bet(100, 4, None); // low 1-18
        bet_check(&mut b, spin(18));
        assert_eq!(b.net, 200);

        let mut b = bet(100, 4, None);
        bet_check(&mut b, spin(19));
        assert_eq!(b.net, -100);

        let mut b = bet(100, 5, None); // high 19-36
        bet_check(&mut b, spin(19));
        assert_eq!(b.net, 200);

        let mut b = bet(100, 5, None);
        bet_check(&mut b, spin(18));
        assert_eq!(b.net, -100);
    }

    #[test]
    fn specific_bet_pays_35_to_1() {
        let mut b = bet(100, 6, Some(17));
        bet_check(&mut b, spin(17));
        assert_eq!(b.net, 100 + 100 * 35);

        let mut b = bet(100, 6, Some(17));
        bet_check(&mut b, spin(18));
        assert_eq!(b.net, -100);
    }

    #[test]
    fn specific_bet_on_zero_is_valid() {
        let mut b = bet(100, 6, Some(0));
        bet_check(&mut b, spin(0));
        assert_eq!(b.net, 100 + 100 * 35);
    }

    #[test]
    fn specific_bet_without_number_loses() {
        let mut b = bet(100, 6, None);
        bet_check(&mut b, spin(17));
        assert_eq!(b.net, -100);
    }

    #[test]
    fn green_bet_wins_on_zero_and_double_zero() {
        let mut b = bet(100, 7, None);
        bet_check(&mut b, spin(0));
        assert_eq!(b.net, 100 + 100 * 17);

        let mut b = bet(100, 7, None);
        bet_check(&mut b, spin(37));
        assert_eq!(b.net, 100 + 100 * 17);

        let mut b = bet(100, 7, None);
        bet_check(&mut b, spin(5));
        assert_eq!(b.net, -100);
    }

    #[test]
    fn even_money_bets_lose_on_zero_and_double_zero() {
        for bet_type in [2, 3, 4, 5] {
            let mut b = bet(100, bet_type, None);
            bet_check(&mut b, spin(0));
            assert_eq!(b.net, -100, "bet_type {} should lose on 0", bet_type);

            let mut b = bet(100, bet_type, None);
            bet_check(&mut b, spin(37));
            assert_eq!(b.net, -100, "bet_type {} should lose on 00", bet_type);
        }
    }

    #[test]
    fn unknown_bet_type_loses() {
        let mut b = bet(100, 99, None);
        bet_check(&mut b, spin(1));
        assert_eq!(b.net, -100);
    }
}
