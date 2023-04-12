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

    return SpinResult {
        value: rng,
        color: get_color(rng),
        oddness: if rng % 2 == 0 { true } else { false },
    };
}

//get the color that was landed on
pub fn get_color(rng: u8) -> Color {
    if rng == 0 || rng == 37 {
        return Color::GREEN;
    }

    if rng <= 10 || (rng <= 28 && rng >= 19) {
        if rng % 2 == 0 {
            return Color::BLACK;
        } else {
            return Color::RED;
        }
    }

    //everything else
    if rng % 2 == 0 {
        Color::RED
    } else {
        Color::BLACK
    }
}

//check all the current bets against the table
pub fn bet_check(bet: &mut BetResult, spin: SpinResult) {
    match bet.bet_type {
        0 => {
            if spin.color != Color::RED {
                bet.net *= -1;
            } else {
                bet.net *= 2;
            }
        }
        1 => {
            if spin.color != Color::BLACK {
                bet.net *= -1;
            } else {
                bet.net *= 2;
            }
        }
        2 => {
            if spin.oddness {
                bet.net *= -1;
            } else {
                bet.net *= 2;
            }
        }
        3 => {
            if !spin.oddness {
                bet.net *= -1;
            } else {
                bet.net *= 2;
            }
        }
        4 => match bet.specific_bet {
            Some(v) => {
                if spin.value == v {
                    bet.net *= 35;
                } else {
                    bet.net *= -1;
                }
            }
            None => {}
        },
        _ => {}
    }
}
