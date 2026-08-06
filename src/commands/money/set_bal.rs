use serenity::{
    model::prelude::interaction::application_command::{
        ApplicationCommandInteraction, CommandDataOptionValue,
    },
    model::prelude::GuildId,
    prelude::Context,
};
use tracing::{trace, warn};

use crate::{
    errors::GenericError,
    utils::money::set_balance,
};

pub async fn handle_set_bal(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
    guild_id: GuildId,
) -> Result<(), GenericError> {
    trace!("set_bal called");

    let caller_member = command
        .member
        .as_ref()
        .ok_or(GenericError::new(&"Could not fetch your member data."))?;

    let mut target: Option<serenity::model::prelude::User> = None;
    let mut amount: Option<i64> = None;

    for option in &command.data.options {
        match option.name.as_str() {
            "user" => {
                if let Some(CommandDataOptionValue::User(user, _)) = option.resolved.as_ref() {
                    target = Some(user.clone());
                }
            }
            "amount" => {
                if let Some(CommandDataOptionValue::Integer(amt)) = option.resolved.as_ref() {
                    amount = Some(*amt);
                }
            }
            _ => {}
        }
    }

    let (target, amount) = match (target, amount) {
        (Some(t), Some(a)) => (t, a),
        _ => {
            return crate::commands::money::respond_ephemeral(
                command,
                ctx,
                "Please provide a valid user and amount.",
            )
            .await;
        }
    };

    if amount < 0 {
        return crate::commands::money::respond_ephemeral(
            command,
            ctx,
            "Amount cannot be negative.",
        )
        .await;
    }

    if target.bot {
        return crate::commands::money::respond_ephemeral(
            command,
            ctx,
            "You cannot set a bot's balance.",
        )
        .await;
    }

    let caller = &command.user;
    if target.id != caller.id {
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
                    "You do not have permission to set this user's balance.",
                )
                .await;
            }
        }
    }

    match set_balance(target.id, guild_id, amount, "set", "system").await {
        Ok(_) => {
            crate::commands::money::respond_ephemeral(
                command,
                ctx,
                &format!("{}'s balance has been set to {}.", target.name, amount),
            )
            .await
        }
        Err(e) => {
            warn!("set_bal failed: {}", e);
            crate::commands::money::respond_ephemeral(
                command,
                ctx,
                "Failed to set user's balance.",
            )
            .await
        }
    }
}
