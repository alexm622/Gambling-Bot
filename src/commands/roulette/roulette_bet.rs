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

    let bet_amount: i64;
    match options.get(0) {
        Some(v) => match v.resolved.as_ref() {
            Some(v) => match v {
                CommandDataOptionValue::Integer(i) => {
                    bet_amount = *i;
                }
                _ => {
                    return Err(String::from("Expected option to be an integer"));
                }
            },
            None => {
                return Err(String::from("Expected option to be an integer"));
            }
        },
        None => {
            return Err(String::from("Expected option to be an integer"));
        }
    };
    let mut bet_type: BettingTypesEnum;

    match options.get(1) {
        Some(v) => match v.resolved.as_ref() {
            Some(v) => match v {
                CommandDataOptionValue::String(s) => {
                    bet_type = BettingTypesEnum::from_str(&s);
                }
                _ => {
                    return Err(String::from("Expected option to be a string"));
                }
            },
            None => {
                return Err(String::from("Expected option to be a string"));
            }
        },
        None => {
            return Err(String::from("Expected option to be a string"));
        }
    };

    //parse for specific bet
    if bet_type.derive_integer() == 8{
        match options.get(2) {
            Some(v) => match v.resolved.as_ref() {
                Some(v) => match v {
                    CommandDataOptionValue::Integer(i) => {
                        bet_type = BettingTypesEnum::Specific(*i as u8);
                    }
                    _ => {
                        return Err(String::from("Expected option to be an integer"));
                    }
                },
                None => {
                    return Err(String::from("Expected option to be an integer"));
                }
            },
            None => {
                return Err(String::from("Expected option to be an integer"));
            }
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
    bet_handler(uid, cid, guild, bet_amount, bet_type.clone(), ctx)
        .await
        .expect("error placing bet");

    let mut embed = serenity::builder::CreateEmbed::default();
    embed.title("Bet Placed!");
    embed.description(format!(
        "You have placed a bet of {} on {}!",
        bet_amount,
        bet_type.to_string()
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
        specific_bet: specific_bet,
    };

    let _res = match insert_roulette_bet(bet).await {
        Err(e) => {
            warn!("unable to place bet {}", e);
            return Ok(());
        }
        Ok(_) => {}
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
    pub fn from_str(bet_type: &str) -> BettingTypesEnum {
        match bet_type {
            "red" => BettingTypesEnum::Red,
            "black" => BettingTypesEnum::Black,
            "green" => BettingTypesEnum::Green,
            "even" => BettingTypesEnum::Even,
            "odd" => BettingTypesEnum::Odd,
            "low" => BettingTypesEnum::Low,
            "high" => BettingTypesEnum::High,
            "specific" => BettingTypesEnum::Specific(0),
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
    pub fn derive_integer(&mut self) -> u8{
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
