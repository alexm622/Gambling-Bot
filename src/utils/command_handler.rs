use serenity::{model::prelude::interaction::application_command::ApplicationCommandInteraction, prelude::Context};
use tracing::info;

use crate::{errors::GenericError, commands::roulette::roulette_command_handler};

pub async fn command_handler(command: ApplicationCommandInteraction, ctx: &Context) -> Result<(), GenericError>{
    let name = command.data.name.clone();
    let category = command_to_category(&name);

    match category{
        CategoriesEnum::Money => {
            //run the command
        },
        CategoriesEnum::Roulette => {
            //send to roulette command handler
            info!("roulette command called");
            roulette_command_handler(command, ctx).await?;
        },
        CategoriesEnum::Poker => {
            //run the command
        },
        CategoriesEnum::Slots => {
            //run the command
        },
        CategoriesEnum::Blackjack => {
            //run the command
        },
        CategoriesEnum::Mod => {
            //run the command
        },
        CategoriesEnum::Help => {
            //run the command
        },
        CategoriesEnum::InvalidCategory => {
            //run the command
        }
    }

    Ok(())
}

//enum of commands categories

pub enum CategoriesEnum {
    Money,
    Roulette,
    Poker,
    Slots,
    Blackjack,
    Mod,
    Help,
    InvalidCategory
}

impl CategoriesEnum{
    pub fn from_str(category: &str) -> CategoriesEnum {
        match category {
            "money" => CategoriesEnum::Money,
            "roulette" => CategoriesEnum::Roulette,
            "poker" => CategoriesEnum::Poker,
            "slots" => CategoriesEnum::Slots,
            "blackjack" => CategoriesEnum::Blackjack,
            "mod" => CategoriesEnum::Mod,
            "help" => CategoriesEnum::Help,
            _ => CategoriesEnum::InvalidCategory
        }
    }
}

fn command_to_category(command: &str) -> CategoriesEnum {
    match command {
        "bal" | "reset_bal" | "reset_user_bal" => CategoriesEnum::Money,
        "roulette" | "roulette_odds" | "roulette_table" => CategoriesEnum::Roulette,
        "poker" => CategoriesEnum::Poker,
        "slots" => CategoriesEnum::Slots,
        "blackjack" => CategoriesEnum::Blackjack,
        _ => CategoriesEnum::InvalidCategory
    }
}