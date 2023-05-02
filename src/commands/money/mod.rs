use crate::errors::GenericError;
use serenity::{
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
    prelude::Context,
};

use tracing::{trace, warn};

pub mod bal;



pub async fn money_command_handler(
    command: ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let name = command.data.name.clone();

    trace!("money command called: {}", name);
    trace!("balance called");
    let guild_id = command.clone().guild_id.unwrap();
    let user = command.clone().user;

    trace!("guild id: {:?}", guild_id);
    trace!("user: {:?}", user);


    match MoneyCommandsEnum::from_str(&name) {
        MoneyCommandsEnum::Balance => {
            
            let embed = bal::get_bal_embed(&command.data.options, guild_id, user).await.expect("error getting balance embed");
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
        MoneyCommandsEnum::InvalidCommand => {
            //run the command
        }
    }

    Ok(())
}

pub enum MoneyCommandsEnum {
    Balance,
    InvalidCommand,
}

impl MoneyCommandsEnum {
    pub fn from_str(command: &str) -> MoneyCommandsEnum {
        match command {
            "bal" => MoneyCommandsEnum::Balance,
            _ => MoneyCommandsEnum::InvalidCommand,
        }
    }
}
