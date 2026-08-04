use serenity::{
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType, MessageFlags,
    },
    prelude::Context,
};

use crate::errors::GenericError;

pub async fn poker_discard_handler(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    respond_ephemeral(
        command,
        ctx,
        "Card discarding is not available in Texas Hold'em.",
    )
    .await
}

async fn respond_ephemeral(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
    content: &str,
) -> Result<(), GenericError> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| m.content(content).flags(MessageFlags::EPHEMERAL))
        })
        .await
        .map_err(|e| GenericError::new(&e.to_string()))
}
