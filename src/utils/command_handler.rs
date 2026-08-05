use serenity::{
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
    prelude::Context,
};
use tracing::info;

use crate::{
    commands::{
        money::money_command_handler, poker::poker_command_handler,
        roulette::roulette_command_handler,
    },
    errors::GenericError,
};

pub async fn command_handler(
    command: ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let name = command.data.name.clone();
    let category = command_to_category(&name);

    match category {
        CategoriesEnum::Money => {
            //send to money command handler
            info!("money command called");
            money_command_handler(command, ctx).await?;
        }
        CategoriesEnum::Roulette => {
            //send to roulette command handler
            info!("roulette command called");
            roulette_command_handler(command, ctx).await?;
        }
        CategoriesEnum::Poker => {
            //send to poker command handler
            poker_command_handler(command, ctx).await?;
        }
        CategoriesEnum::Slots => {
            //run the command
        }
        CategoriesEnum::Blackjack => {
            //run the command
        }
        CategoriesEnum::Mod => {
            //run the command
        }
        CategoriesEnum::Help => {
            //run the command
        }
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
    InvalidCategory,
}

impl CategoriesEnum {
    pub fn from_name(category: &str) -> CategoriesEnum {
        match category {
            "money" => CategoriesEnum::Money,
            "roulette" => CategoriesEnum::Roulette,
            "poker" => CategoriesEnum::Poker,
            "slots" => CategoriesEnum::Slots,
            "blackjack" => CategoriesEnum::Blackjack,
            "mod" => CategoriesEnum::Mod,
            "help" => CategoriesEnum::Help,
            _ => CategoriesEnum::InvalidCategory,
        }
    }
}

fn command_to_category(command: &str) -> CategoriesEnum {
    match command {
        "bal" | "reset_bal" | "reset_user_bal" => CategoriesEnum::Money,
        "roulette" | "roulette_odds" | "roulette_table" => CategoriesEnum::Roulette,
        "pstart" | "pjoin" | "pleave" | "phand" => CategoriesEnum::Poker,
        "slots" => CategoriesEnum::Slots,
        "blackjack" => CategoriesEnum::Blackjack,
        _ => CategoriesEnum::InvalidCategory,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_route_to_their_category() {
        for cmd in ["bal", "reset_bal", "reset_user_bal"] {
            assert!(
                matches!(command_to_category(cmd), CategoriesEnum::Money),
                "{}",
                cmd
            );
        }
        for cmd in ["roulette", "roulette_odds", "roulette_table"] {
            assert!(
                matches!(command_to_category(cmd), CategoriesEnum::Roulette),
                "{}",
                cmd
            );
        }
        for cmd in ["pstart", "pjoin", "pleave", "phand"] {
            assert!(
                matches!(command_to_category(cmd), CategoriesEnum::Poker),
                "{}",
                cmd
            );
        }
        assert!(matches!(
            command_to_category("slots"),
            CategoriesEnum::Slots
        ));
        assert!(matches!(
            command_to_category("blackjack"),
            CategoriesEnum::Blackjack
        ));
    }

    #[test]
    fn unknown_commands_are_invalid() {
        for cmd in ["", "pdance", "BAL", "roulette_"] {
            assert!(
                matches!(command_to_category(cmd), CategoriesEnum::InvalidCategory),
                "{}",
                cmd
            );
        }
    }

    #[test]
    fn category_from_str() {
        assert!(matches!(
            CategoriesEnum::from_name("poker"),
            CategoriesEnum::Poker
        ));
        assert!(matches!(
            CategoriesEnum::from_name("nope"),
            CategoriesEnum::InvalidCategory
        ));
    }
}
