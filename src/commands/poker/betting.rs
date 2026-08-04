use serenity::{
    model::prelude::interaction::{
        application_command::{
            ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
        },
        InteractionResponseType, MessageFlags,
    },
    prelude::Context,
};

use crate::{
    errors::GenericError,
    redis::{poker, users},
};

pub async fn poker_fold(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let guild_id = command
        .guild_id
        .ok_or(GenericError::new(&"Guild ID not found"))?;
    let cid = command.channel_id;
    let uid = command.user.id;

    if !ensure_in_game(command, ctx, guild_id, cid, uid).await? {
        return Ok(());
    }

    poker::fold_user(guild_id, cid, uid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    respond_ephemeral(command, ctx, "You have folded.").await?;
    Ok(())
}

pub async fn poker_check(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let guild_id = command
        .guild_id
        .ok_or(GenericError::new(&"Guild ID not found"))?;
    let cid = command.channel_id;
    let uid = command.user.id;

    if !ensure_in_game(command, ctx, guild_id, cid, uid).await? {
        return Ok(());
    }

    let current_bet = poker::get_current_bet(guild_id, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    let user_bet = poker::get_user_bet(guild_id, cid, uid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    if user_bet < current_bet {
        respond_ephemeral(
            command,
            ctx,
            &format!(
                "You cannot check. The current bet is {} and you have only put in {}.",
                current_bet, user_bet
            ),
        )
        .await?;
        return Ok(());
    }

    respond_ephemeral(command, ctx, "You have checked.").await?;
    Ok(())
}

pub async fn poker_call(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let guild_id = command
        .guild_id
        .ok_or(GenericError::new(&"Guild ID not found"))?;
    let cid = command.channel_id;
    let uid = command.user.id;

    if !ensure_in_game(command, ctx, guild_id, cid, uid).await? {
        return Ok(());
    }

    let current_bet = poker::get_current_bet(guild_id, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    let user_bet = poker::get_user_bet(guild_id, cid, uid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    let to_call = current_bet.saturating_sub(user_bet);

    if to_call == 0 {
        respond_ephemeral(
            command,
            ctx,
            "There is no bet to call. You can `/pcheck` instead.",
        )
        .await?;
        return Ok(());
    }

    let bal = users::get_user_bal(uid, guild_id)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    if (bal as u64) < to_call {
        respond_ephemeral(command, ctx, "You do not have enough chips to call.").await?;
        return Ok(());
    }

    users::user_add(uid, guild_id, -(to_call as i64))
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    poker::add_to_pot(guild_id, cid, to_call)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    poker::set_user_bet(guild_id, cid, uid, current_bet)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    respond_ephemeral(
        command,
        ctx,
        &format!("You have called the {} chip bet.", current_bet),
    )
    .await?;
    Ok(())
}

pub async fn poker_raise(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let guild_id = command
        .guild_id
        .ok_or(GenericError::new(&"Guild ID not found"))?;
    let cid = command.channel_id;
    let uid = command.user.id;

    if !ensure_in_game(command, ctx, guild_id, cid, uid).await? {
        return Ok(());
    }

    let amount = match get_integer_option(&command.data.options, "bet") {
        Some(v) if v > 0 => v as u64,
        _ => {
            respond_ephemeral(command, ctx, "Please provide a positive bet amount.").await?;
            return Ok(());
        }
    };

    let current_bet = poker::get_current_bet(guild_id, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    let user_bet = poker::get_user_bet(guild_id, cid, uid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    let new_bet = current_bet + amount;
    let to_pay = new_bet.saturating_sub(user_bet);

    let bal = users::get_user_bal(uid, guild_id)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    if (bal as u64) < to_pay {
        respond_ephemeral(
            command,
            ctx,
            "You do not have enough chips to raise by that amount.",
        )
        .await?;
        return Ok(());
    }

    users::user_add(uid, guild_id, -(to_pay as i64))
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    poker::add_to_pot(guild_id, cid, to_pay)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    poker::set_current_bet(guild_id, cid, new_bet)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    poker::set_user_bet(guild_id, cid, uid, new_bet)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    respond_ephemeral(
        command,
        ctx,
        &format!(
            "You have raised the bet by {} chips. The current bet is now {}.",
            amount, new_bet
        ),
    )
    .await?;
    Ok(())
}

pub async fn poker_allin(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let guild_id = command
        .guild_id
        .ok_or(GenericError::new(&"Guild ID not found"))?;
    let cid = command.channel_id;
    let uid = command.user.id;

    if !ensure_in_game(command, ctx, guild_id, cid, uid).await? {
        return Ok(());
    }

    let bal = users::get_user_bal(uid, guild_id)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    if bal <= 0 {
        respond_ephemeral(command, ctx, "You have no chips to go all in with.").await?;
        return Ok(());
    }

    let amount = bal as u64;
    let current_bet = poker::get_current_bet(guild_id, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    let user_bet = poker::get_user_bet(guild_id, cid, uid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    let new_bet = current_bet + amount;
    let to_pay = amount + current_bet.saturating_sub(user_bet);

    if (bal as u64) < to_pay {
        respond_ephemeral(command, ctx, "You do not have enough chips to go all in.").await?;
        return Ok(());
    }

    users::user_add(uid, guild_id, -(to_pay as i64))
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    poker::add_to_pot(guild_id, cid, to_pay)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    poker::set_current_bet(guild_id, cid, new_bet)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    poker::set_user_bet(guild_id, cid, uid, new_bet)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    respond_ephemeral(
        command,
        ctx,
        &format!("You have gone all in with {} chips!", to_pay),
    )
    .await?;
    Ok(())
}

async fn ensure_in_game(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
    guild_id: serenity::model::prelude::GuildId,
    cid: serenity::model::prelude::ChannelId,
    uid: serenity::model::prelude::UserId,
) -> Result<bool, GenericError> {
    if !poker::is_joinned(uid, guild_id, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
    {
        respond_ephemeral(command, ctx, "You are not in this poker game.").await?;
        return Ok(false);
    }
    Ok(true)
}

fn get_integer_option(options: &[CommandDataOption], name: &str) -> Option<i64> {
    for opt in options {
        if opt.name == name {
            if let Some(CommandDataOptionValue::Integer(v)) = opt.resolved.as_ref() {
                return Some(*v);
            }
        }
    }
    None
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
