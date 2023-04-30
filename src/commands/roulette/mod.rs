use serenity::{
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
    prelude::Context,
};

use tracing::{trace, warn};

use crate::errors::GenericError;

pub mod roulette_bet;
pub mod roulette_odds;


pub async fn roulette_command_handler(
    command: ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let name = command.data.name.clone();

    match RouletteCommandsEnum::from_str(&name) {
        // place a bet
        RouletteCommandsEnum::Roulette => {
            trace!("roulette called");
            let embed = roulette_bet::get_bet_embed(&command.data.options, command.user.id, command.channel_id, command.guild_id.unwrap(), ctx).await.map_err(|e| {
                warn!("error getting bet embed: {}", e);
                return GenericError::new(&format!("error getting bet embed: {}", e));
            })?;
            match command.create_interaction_response(ctx, |response| {
                response.kind(serenity::model::prelude::interaction::InteractionResponseType::ChannelMessageWithSource);
                response.interaction_response_data(|message| {
                    message.add_embed(embed)
                })
            }).await{
                Ok(_) => {return Ok(())},
                Err(e) => {
                    warn!("error sending response: {}", e);
                    return Err(GenericError::new(&format!("error sending response: {}", e)));
                }
            }         
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
                    warn!("error sending response: {}", e);
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
