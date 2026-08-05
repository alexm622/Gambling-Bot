use serenity::{
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType, MessageFlags,
    },
    prelude::Context,
};
use tracing::{trace, warn};

use crate::errors::GenericError;

pub mod bot;
pub mod components;
pub mod flow;
pub mod game;
pub mod hand;
pub mod hand_evaluator;
pub mod session;
pub mod ui;

pub async fn poker_command_handler(
    command: ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let name = command.data.name.clone();

    trace!("poker command called: {}", name);

    match PokerCommandsEnum::from_name(&name) {
        PokerCommandsEnum::Hand => hand::poker_hand_handler(&command, ctx).await,
        PokerCommandsEnum::Join => {
            // keep slash join as a fallback to the button
            let guild_id = command
                .guild_id
                .ok_or(GenericError::new(&"Guild ID not found"))?;
            flow::handle_join(ctx, guild_id, command.channel_id, command.user.id).await?;
            Ok(())
        }
        PokerCommandsEnum::Leave => game::poker_leave(&command, ctx).await,
        PokerCommandsEnum::Start => game::poker_start(&command, ctx).await,
        PokerCommandsEnum::InvalidCommand => {
            warn!("invalid poker command called: {}", name);
            respond_ephemeral(&command, ctx, "Invalid poker command").await
        }
    }
}

pub async fn respond_ephemeral(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
    content: &str,
) -> Result<(), GenericError> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content(content).flags(MessageFlags::EPHEMERAL)
                })
        })
        .await
        .map_err(|e| GenericError::new(&e.to_string()))
}

pub enum PokerCommandsEnum {
    Join,
    Leave,
    Start,
    Hand,
    InvalidCommand,
}

impl PokerCommandsEnum {
    pub fn from_name(command: &str) -> PokerCommandsEnum {
        match command {
            "pjoin" => PokerCommandsEnum::Join,
            "pstart" => PokerCommandsEnum::Start,
            "pleave" => PokerCommandsEnum::Leave,
            "phand" => PokerCommandsEnum::Hand,
            _ => PokerCommandsEnum::InvalidCommand,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            PokerCommandsEnum::Join => "pjoin",
            PokerCommandsEnum::Leave => "pleave",
            PokerCommandsEnum::Start => "pstart",
            PokerCommandsEnum::Hand => "phand",
            PokerCommandsEnum::InvalidCommand => "invalid",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_strings_roundtrip() {
        for cmd in [
            PokerCommandsEnum::Join,
            PokerCommandsEnum::Leave,
            PokerCommandsEnum::Start,
            PokerCommandsEnum::Hand,
        ] {
            let parsed = PokerCommandsEnum::from_name(cmd.to_str());
            assert_eq!(parsed.to_str(), cmd.to_str());
        }
    }

    #[test]
    fn unknown_command_is_invalid() {
        assert_eq!(
            PokerCommandsEnum::from_name("pdance").to_str(),
            PokerCommandsEnum::InvalidCommand.to_str()
        );
    }
}
