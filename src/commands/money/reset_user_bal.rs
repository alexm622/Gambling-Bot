use serenity::{
    model::prelude::interaction::{
        application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
        message_component::MessageComponentInteraction,
    },
    model::prelude::GuildId,
    prelude::Context,
};
use tracing::{trace, warn};

use crate::{
    commands::money::respond_component_ephemeral,
    errors::GenericError,
    utils::money::reset_balance,
};

pub async fn handle_reset_user_bal(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
    guild_id: GuildId,
) -> Result<(), GenericError> {
    trace!("reset_user_bal called");

    let caller = &command.user;
    let caller_member = command
        .member
        .as_ref()
        .ok_or(GenericError::new(&"Could not fetch your member data."))?;

    let target = match command
        .data
        .options
        .iter()
        .find(|o| o.name == "user")
        .and_then(|o| o.resolved.as_ref())
    {
        Some(CommandDataOptionValue::User(user, _)) => user.clone(),
        _ => return crate::commands::money::respond_ephemeral(command, ctx, "Invalid user.").await,
    };

    if target.id == caller.id {
        return crate::commands::money::respond_ephemeral(
            command,
            ctx,
            "You cannot reset your own balance with this command. Use /reset_bal instead.",
        )
        .await;
    }

    if target.bot {
        return crate::commands::money::respond_ephemeral(
            command,
            ctx,
            "You cannot reset a bot's balance.",
        )
        .await;
    }

    let target_member = match guild_id.member(&ctx.http, target.id).await {
        Ok(m) => m,
        Err(_) => {
            return crate::commands::money::respond_ephemeral(
                command,
                ctx,
                "That user is not in this server.",
            )
            .await;
        }
    };

    // Caller must be a moderator: have ADMINISTRATOR or a higher role position than the target.
    let caller_permissions = caller_member.permissions.unwrap_or_default();
    let caller_is_admin = caller_permissions.administrator();
    let caller_highest_role = caller_member.highest_role_info(&ctx.cache);
    let target_highest_role = target_member.highest_role_info(&ctx.cache);

    if !caller_is_admin {
        let can_moderate = match (caller_highest_role, target_highest_role) {
            (Some((caller_pos, _)), Some((target_pos, _))) => caller_pos > target_pos,
            (Some(_), None) => true,
            _ => false,
        };
        if !can_moderate {
            return crate::commands::money::respond_ephemeral(
                command,
                ctx,
                "You do not have permission to reset this user's balance.",
            )
            .await;
        }
    }

    match reset_balance(target.id, guild_id).await {
        Ok(new_balance) => {
            crate::commands::money::respond_ephemeral(
                command,
                ctx,
                &format!(
                    "{}'s balance has been reset to {}.",
                    target.name, new_balance
                ),
            )
            .await
        }
        Err(e) => {
            warn!("reset_user_bal failed: {}", e);
            crate::commands::money::respond_ephemeral(
                command,
                ctx,
                "Failed to reset user's balance.",
            )
            .await
        }
    }
}

pub async fn handle_component(
    component: &MessageComponentInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let _ = (component, ctx);
    respond_component_ephemeral(component, ctx, "Unknown component.").await
}
