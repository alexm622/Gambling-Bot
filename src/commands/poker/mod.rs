use serenity::{
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType, MessageFlags,
    },
    prelude::Context,
};
use tracing::{trace, warn};

use crate::errors::GenericError;

pub mod betting;
pub mod bot;
pub mod components;
pub mod flow;
pub mod game;
pub mod hand_evaluator;
pub mod poker_discard;
pub mod poker_draw;
pub mod session;
pub mod ui;

pub async fn poker_command_handler(
    command: ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let name = command.data.name.clone();

    trace!("poker command called: {}", name);

    match PokerCommandsEnum::from_str(&name) {
        PokerCommandsEnum::Check => {
            // deprecated: now handled by buttons
            respond_deprecated(&command, ctx).await
        }
        PokerCommandsEnum::Call => respond_deprecated(&command, ctx).await,
        PokerCommandsEnum::Discard => poker_discard::poker_discard_handler(&command, ctx).await,
        PokerCommandsEnum::Draw => poker_draw::poker_draw_handler(&command, ctx).await,
        PokerCommandsEnum::Hand => poker_draw::poker_hand_handler(&command, ctx).await,
        PokerCommandsEnum::Fold => respond_deprecated(&command, ctx).await,
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
        PokerCommandsEnum::Raise => respond_deprecated(&command, ctx).await,
        PokerCommandsEnum::AllIn => respond_deprecated(&command, ctx).await,
        PokerCommandsEnum::InvalidCommand => {
            warn!("invalid poker command called: {}", name);
            command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .content("Invalid poker command")
                                .flags(MessageFlags::EPHEMERAL)
                        })
                })
                .await
                .map_err(|e| GenericError::new(&e.to_string()))
        }
    }
}

async fn respond_deprecated(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message
                        .content("This action is now handled through the game buttons.")
                        .flags(MessageFlags::EPHEMERAL)
                })
        })
        .await
        .map_err(|e| GenericError::new(&e.to_string()))
}

pub enum PokerCommandsEnum {
    Join,
    Leave,
    Start,
    Draw,
    Hand,
    Discard,
    Fold,
    Raise,
    Check,
    Call,
    AllIn,
    InvalidCommand,
}

impl PokerCommandsEnum {
    pub fn from_str(command: &str) -> PokerCommandsEnum {
        match command {
            "pjoin" => PokerCommandsEnum::Join,
            "pstart" => PokerCommandsEnum::Start,
            "pleave" => PokerCommandsEnum::Leave,
            "pallin" => PokerCommandsEnum::AllIn,
            "pdraw" => PokerCommandsEnum::Draw,
            "phand" => PokerCommandsEnum::Hand,
            "pdiscard" => PokerCommandsEnum::Discard,
            "pfold" => PokerCommandsEnum::Fold,
            "praise" => PokerCommandsEnum::Raise,
            "pcheck" => PokerCommandsEnum::Check,
            "pcall" => PokerCommandsEnum::Call,
            _ => PokerCommandsEnum::InvalidCommand,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            PokerCommandsEnum::Join => "pjoin",
            PokerCommandsEnum::Leave => "pleave",
            PokerCommandsEnum::Start => "pstart",
            PokerCommandsEnum::Draw => "pdraw",
            PokerCommandsEnum::Hand => "phand",
            PokerCommandsEnum::Discard => "pdiscard",
            PokerCommandsEnum::Fold => "pfold",
            PokerCommandsEnum::Raise => "praise",
            PokerCommandsEnum::AllIn => "pallin",
            PokerCommandsEnum::Check => "pcheck",
            PokerCommandsEnum::Call => "pcall",
            PokerCommandsEnum::InvalidCommand => "invalid",
        }
    }
}
