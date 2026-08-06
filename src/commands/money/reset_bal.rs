use std::time::{SystemTime, UNIX_EPOCH};

use serenity::{
    builder::{CreateActionRow, CreateButton},
    model::application::component::ButtonStyle,
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction,
        message_component::MessageComponentInteraction,
        InteractionResponseType, MessageFlags,
    },
    model::prelude::{GuildId, UserId},
    prelude::Context,
};
use tracing::{trace, warn};

use crate::{
    commands::money::respond_component_ephemeral,
    errors::GenericError,
    utils::money::reset_balance,
};

const CONFIRM_TIMEOUT_SECONDS: u64 = 300;

pub async fn handle_reset_bal(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
    _guild_id: GuildId,
    user_id: UserId,
) -> Result<(), GenericError> {
    trace!("reset_bal called for user {}", user_id.0);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let custom_id = format!("reset_bal:{}:{}", user_id.0, timestamp);

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message
                        .content("Are you sure you want to reset your balance?")
                        .flags(MessageFlags::EPHEMERAL)
                        .components(|c| {
                            let mut row = CreateActionRow::default();
                            row.add_button(
                                CreateButton::default()
                                    .custom_id(&custom_id)
                                    .label("Confirm")
                                    .style(ButtonStyle::Danger)
                                    .to_owned(),
                            );
                            c.add_action_row(row);
                            c
                        })
                })
        })
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    Ok(())
}

pub async fn handle_component(
    component: &MessageComponentInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let parts: Vec<&str> = component.data.custom_id.split(':').collect();
    if parts.len() != 3 {
        return respond_component_ephemeral(component, ctx, "Invalid confirmation.").await;
    }

    let user_id = parts[1]
        .parse::<u64>()
        .map_err(|e| GenericError::new(&e.to_string()))?;
    let created_at = parts[2]
        .parse::<u64>()
        .map_err(|e| GenericError::new(&e.to_string()))?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if now.saturating_sub(created_at) > CONFIRM_TIMEOUT_SECONDS {
        return respond_component_ephemeral(component, ctx, "This confirmation has expired.").await;
    }

    if component.user.id.0 != user_id {
        return respond_component_ephemeral(component, ctx, "This confirmation is not for you.").await;
    }

    let Some(guild_id) = component.guild_id else {
        return respond_component_ephemeral(component, ctx, "This must be used in a server.").await;
    };

    match reset_balance(component.user.id, guild_id).await {
        Ok(new_balance) => {
            // disable the button
            let _ = component
                .message
                .clone()
                .edit(&ctx.http, |m| {
                    m.content(format!(
                        "Balance reset. Your new balance is {}.",
                        new_balance
                    ))
                    .components(|c| c)
                })
                .await;
            respond_component_ephemeral(
                component,
                ctx,
                &format!("Your balance has been reset to {}.", new_balance),
            )
            .await
        }
        Err(e) => {
            warn!("reset_bal failed: {}", e);
            respond_component_ephemeral(component, ctx, "Failed to reset balance.").await
        }
    }
}
