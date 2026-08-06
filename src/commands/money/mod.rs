use crate::{errors::GenericError, secrets::is_admin_user};
use serenity::{
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction,
        message_component::MessageComponentInteraction,
        InteractionResponseType, MessageFlags,
    },
    prelude::Context,
};

use tracing::trace;

pub mod bal;
pub mod reset_bal;
pub mod reset_user_bal;
pub mod set_bal;

pub async fn money_command_handler(
    command: ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let name = command.data.name.clone();

    trace!("money command called: {}", name);

    if !is_admin_user(command.user.id) {
        return respond_ephemeral(&command, ctx, "You are not authorized to use this command.").await;
    }

    let guild_id = command
        .guild_id
        .ok_or(GenericError::new(&"This command must be run in a server."))?;
    let user = command.user.clone();

    trace!("guild id: {:?}", guild_id);
    trace!("user: {:?}", user);

    match MoneyCommandsEnum::from_name(&name) {
        MoneyCommandsEnum::Balance => {
            let embed = bal::get_bal_embed(&command.data.options, guild_id, user)
                .await
                .map_err(|e| GenericError::new(&e))?;
            command
                .create_interaction_response(ctx, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.add_embed(embed))
                })
                .await
                .map_err(|e| GenericError::new(&e.to_string()))?;
            Ok(())
        }
        MoneyCommandsEnum::ResetBalance => {
            reset_bal::handle_reset_bal(&command, ctx, guild_id, user.id).await
        }
        MoneyCommandsEnum::ResetUserBalance => {
            reset_user_bal::handle_reset_user_bal(&command, ctx, guild_id).await
        }
        MoneyCommandsEnum::SetBalance => {
            set_bal::handle_set_bal(&command, ctx, guild_id).await
        }
        MoneyCommandsEnum::InvalidCommand => {
            respond_ephemeral(&command, ctx, "Unknown money command").await
        }
    }
}

pub async fn handle_money_component(
    component: &MessageComponentInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    if component.data.custom_id.starts_with("reset_bal:") {
        return reset_bal::handle_component(component, ctx).await;
    }

    if component.data.custom_id.starts_with("reset_user_bal:") {
        return reset_user_bal::handle_component(component, ctx).await;
    }

    Ok(())
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

pub async fn respond_component_ephemeral(
    component: &MessageComponentInteraction,
    ctx: &Context,
    content: &str,
) -> Result<(), GenericError> {
    component
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

pub enum MoneyCommandsEnum {
    Balance,
    ResetBalance,
    ResetUserBalance,
    SetBalance,
    InvalidCommand,
}

impl MoneyCommandsEnum {
    pub fn from_name(command: &str) -> MoneyCommandsEnum {
        match command {
            "bal" => MoneyCommandsEnum::Balance,
            "reset_bal" => MoneyCommandsEnum::ResetBalance,
            "reset_user_bal" => MoneyCommandsEnum::ResetUserBalance,
            "set_bal" => MoneyCommandsEnum::SetBalance,
            _ => MoneyCommandsEnum::InvalidCommand,
        }
    }
}
