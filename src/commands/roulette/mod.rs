use serenity::{
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
    prelude::Context,
};

use tracing::trace;

use crate::errors::GenericError;

pub mod roulette_odds;

pub async fn roulette_command_handler(
    command: ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let name = command.data.name.clone();

    match RouletteCommandsEnum::from_str(&name) {
        RouletteCommandsEnum::Roulette => {
            
        }
        RouletteCommandsEnum::RouletteOdds => {
            trace!("roulette odds called");
            let embed = roulette_odds::get_odds_embed(&command.data.options);
            match command.create_interaction_response(ctx, |response| {
                response.kind(serenity::model::prelude::interaction::InteractionResponseType::ChannelMessageWithSource);
                response.interaction_response_data(|message| {
                    message.add_embed(embed)
                })
            }).await{
                Ok(_) => {return Ok(())},
                Err(e) => {
                    println!("error sending response: {}", e);
                    return Err(GenericError::new(&format!("error sending response: {}", e)));
                }
            }
        }
        RouletteCommandsEnum::RouletteTable => {
            //run the command
        }
        RouletteCommandsEnum::InvalidCommand => {
            //run the command
        }
    }

    Ok(())
}

//enum of commands

pub enum RouletteCommandsEnum {
    Roulette,
    RouletteOdds,
    RouletteTable,
    InvalidCommand,
}

impl RouletteCommandsEnum {
    pub fn from_str(command: &str) -> RouletteCommandsEnum {
        match command {
            "roulette" => RouletteCommandsEnum::Roulette,
            "roulette_odds" => RouletteCommandsEnum::RouletteOdds,
            "roulette_table" => RouletteCommandsEnum::RouletteTable,
            _ => RouletteCommandsEnum::InvalidCommand,
        }
    }
}
