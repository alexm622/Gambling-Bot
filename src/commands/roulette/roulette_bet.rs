use serde::{Deserialize, Serialize};
use serenity::model::prelude::{
    interaction::application_command::{CommandDataOption, CommandDataOptionValue},
    ChannelId, GuildId, UserId,
};
use tracing::warn;

use crate::{
    redis::roulette::spin_table,
    sql::{
        insert::insert_roulette_bet,
        structs::{BettingTypes, RouletteBet},
    },
};

pub async fn get_bet_embed(
    options: &[CommandDataOption],
    uid: UserId,
    cid: ChannelId,
    guild: GuildId,
    ctx: &serenity::client::Context,
) -> Result<serenity::builder::CreateEmbed, String> {
    //get bet amount
    let bet_amount: i64 = match options.first().and_then(|v| v.resolved.as_ref()) {
        Some(CommandDataOptionValue::Integer(i)) => *i,
        _ => return Err(String::from("Expected option to be an integer")),
    };

    let mut bet_type: BettingTypesEnum = match options.get(1).and_then(|v| v.resolved.as_ref()) {
        Some(CommandDataOptionValue::String(s)) => BettingTypesEnum::from_name(s),
        _ => return Err(String::from("Expected option to be a string")),
    };

    //parse for specific bet
    if bet_type.derive_integer() == 8 {
        bet_type = match options.get(2).and_then(|v| v.resolved.as_ref()) {
            Some(CommandDataOptionValue::Integer(i)) => {
                if (0..=36).contains(i) {
                    BettingTypesEnum::Specific(*i as u8)
                } else {
                    BettingTypesEnum::Invalid
                }
            }
            _ => return Err(String::from("Expected option to be an integer")),
        };
    }

    if bet_type == BettingTypesEnum::Invalid {
        //create a fail embed
        let mut embed = serenity::builder::CreateEmbed::default();
        embed.title("Invalid Bet Type");
        embed.description("The bet type you have entered is invalid. Please try again.");
        embed.color(serenity::utils::Colour::from_rgb(255, 0, 0));
        return Ok(embed);
    }

    //this below halts
    bet_handler(uid, cid, guild, bet_amount, bet_type, ctx)
        .await
        .expect("error placing bet");

    let mut embed = serenity::builder::CreateEmbed::default();
    embed.title("Bet Placed!");
    embed.description(format!(
        "You have placed a bet of {} on {}!",
        bet_amount, bet_type
    ));
    embed.color(serenity::utils::Colour::from_rgb(255, 0, 0));
    Ok(embed)
}

pub async fn bet_handler(
    uid: UserId,
    cid: ChannelId,
    guild: GuildId,
    bet: i64,
    bet_type: BettingTypesEnum,
    ctx: &serenity::client::Context,
) -> Result<(), String> {
    let specific_bet: Option<u8> = bet_type.get_specific();

    let bet: RouletteBet = RouletteBet {
        amount: bet,
        user_id: uid,
        channel_id: cid,
        guild_id: guild,
        bet_type: BettingTypes::from_bettingtypeenum(bet_type),
        specific_bet,
    };

    if let Err(e) = insert_roulette_bet(bet).await {
        warn!("unable to place bet {}", e);
        return Ok(());
    };

    let _spin = spin_table(guild, cid, ctx.clone()).await;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum BettingTypesEnum {
    Red,
    Black,
    Green,
    Even,
    Odd,
    Low,
    High,
    Specific(u8),
    Invalid,
}

impl BettingTypesEnum {
    pub fn from_name(bet_type: &str) -> BettingTypesEnum {
        match bet_type {
            "red" => BettingTypesEnum::Red,
            "black" => BettingTypesEnum::Black,
            "green" => BettingTypesEnum::Green,
            "even" => BettingTypesEnum::Even,
            "odd" => BettingTypesEnum::Odd,
            "low" => BettingTypesEnum::Low,
            "high" => BettingTypesEnum::High,
            // "number" is what the /roulette slash command sends for single-number bets
            "specific" | "number" => BettingTypesEnum::Specific(0),
            _ => match bet_type.parse::<u8>() {
                Ok(v) => {
                    if v > 36 {
                        BettingTypesEnum::Invalid
                    } else {
                        BettingTypesEnum::Specific(v)
                    }
                }
                Err(_) => BettingTypesEnum::Invalid,
            },
        }
    }

    pub fn get_specific(&self) -> Option<u8> {
        match self {
            BettingTypesEnum::Specific(v) => Some(*v),
            _ => None,
        }
    }

    pub fn set_specific(&mut self, v: u8) {
        *self = BettingTypesEnum::Specific(v);
    }

    //return a unique id for the bet type
    pub fn derive_integer(&mut self) -> u8 {
        match self {
            BettingTypesEnum::Red => 1,
            BettingTypesEnum::Black => 2,
            BettingTypesEnum::Green => 3,
            BettingTypesEnum::Even => 4,
            BettingTypesEnum::Odd => 5,
            BettingTypesEnum::Low => 6,
            BettingTypesEnum::High => 7,
            BettingTypesEnum::Specific(_v) => 8,
            BettingTypesEnum::Invalid => 0,
        }
    }
}

impl std::fmt::Display for BettingTypesEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BettingTypesEnum::Red => write!(f, "Red"),
            BettingTypesEnum::Black => write!(f, "Black"),
            BettingTypesEnum::Green => write!(f, "Green"),
            BettingTypesEnum::Even => write!(f, "Even"),
            BettingTypesEnum::Odd => write!(f, "Odd"),
            BettingTypesEnum::Low => write!(f, "Low"),
            BettingTypesEnum::High => write!(f, "High"),
            BettingTypesEnum::Specific(v) => write!(f, "{}", v),
            BettingTypesEnum::Invalid => write!(f, "Invalid"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_name_parses_all_slash_command_choices() {
        // these are the exact strings offered by the /roulette command registration
        assert_eq!(BettingTypesEnum::from_name("red"), BettingTypesEnum::Red);
        assert_eq!(
            BettingTypesEnum::from_name("black"),
            BettingTypesEnum::Black
        );
        assert_eq!(
            BettingTypesEnum::from_name("green"),
            BettingTypesEnum::Green
        );
        assert_eq!(BettingTypesEnum::from_name("even"), BettingTypesEnum::Even);
        assert_eq!(BettingTypesEnum::from_name("odd"), BettingTypesEnum::Odd);
        assert_eq!(BettingTypesEnum::from_name("low"), BettingTypesEnum::Low);
        assert_eq!(BettingTypesEnum::from_name("high"), BettingTypesEnum::High);
        assert_eq!(
            BettingTypesEnum::from_name("number"),
            BettingTypesEnum::Specific(0)
        );
    }

    #[test]
    fn from_name_parses_numbers() {
        assert_eq!(
            BettingTypesEnum::from_name("0"),
            BettingTypesEnum::Specific(0)
        );
        assert_eq!(
            BettingTypesEnum::from_name("17"),
            BettingTypesEnum::Specific(17)
        );
        assert_eq!(
            BettingTypesEnum::from_name("36"),
            BettingTypesEnum::Specific(36)
        );
    }

    #[test]
    fn from_name_rejects_invalid() {
        for s in ["37", "abc", "", "-1"] {
            assert_eq!(
                BettingTypesEnum::from_name(s),
                BettingTypesEnum::Invalid,
                "expected {} to be invalid",
                s
            );
        }
    }

    #[test]
    fn betting_types_map_to_bet_check_ids() {
        // bet_check in utils::roulette matches on these exact u8 values
        assert_eq!(
            BettingTypes::from_bettingtypeenum(BettingTypesEnum::Red) as u8,
            0
        );
        assert_eq!(
            BettingTypes::from_bettingtypeenum(BettingTypesEnum::Black) as u8,
            1
        );
        assert_eq!(
            BettingTypes::from_bettingtypeenum(BettingTypesEnum::Even) as u8,
            2
        );
        assert_eq!(
            BettingTypes::from_bettingtypeenum(BettingTypesEnum::Odd) as u8,
            3
        );
        assert_eq!(
            BettingTypes::from_bettingtypeenum(BettingTypesEnum::Low) as u8,
            4
        );
        assert_eq!(
            BettingTypes::from_bettingtypeenum(BettingTypesEnum::High) as u8,
            5
        );
        assert_eq!(
            BettingTypes::from_bettingtypeenum(BettingTypesEnum::Specific(7)) as u8,
            6
        );
        assert_eq!(
            BettingTypes::from_bettingtypeenum(BettingTypesEnum::Green) as u8,
            7
        );
    }
}
