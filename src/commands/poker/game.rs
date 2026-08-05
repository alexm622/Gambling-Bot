use serenity::{
    builder::CreateEmbed,
    model::prelude::interaction::{
        application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
        InteractionResponseType,
    },
    prelude::Context,
};
use tracing::trace;

use crate::{
    commands::poker::{flow, respond_ephemeral, session},
    errors::GenericError,
    redis::poker,
};

pub async fn poker_start(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let guild_id = command
        .guild_id
        .ok_or(GenericError::new(&"Guild ID not found"))?;
    let cid = command.channel_id;
    let uid = command.user.id;

    trace!(
        "starting poker game in guild {:?} channel {:?}",
        guild_id,
        cid
    );

    if let Some(state) = session::load_state(guild_id, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
    {
        if state.phase != session::Phase::Finished {
            respond_ephemeral(
                command,
                ctx,
                "A poker game is already active in this channel.",
            )
            .await?;
            return Ok(());
        }
    }

    // clean up any stale game data for this channel
    poker::cleanup_game(guild_id, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    let max_hand_bet = get_integer_option(&command.data.options, "max_bet").map(|v| v as u64);
    if let Some(max) = max_hand_bet {
        if max == 0 {
            respond_ephemeral(command, ctx, "Maximum hand bet must be greater than 0.").await?;
            return Ok(());
        }
        let bal = crate::redis::users::get_user_bal(uid, guild_id)
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
        if (bal as u64) < max {
            respond_ephemeral(
                command,
                ctx,
                "Maximum hand bet cannot exceed your current balance.",
            )
            .await?;
            return Ok(());
        }
    }

    flow::start_lobby(ctx, guild_id, cid, uid, max_hand_bet).await?;

    let embed = CreateEmbed::default()
        .title("Poker Lobby Opened")
        .description(format!(
            "<@{}> has opened a poker lobby! Click Join to play.",
            uid
        ))
        .color(serenity::utils::Colour::DARK_GREEN)
        .to_owned();

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| m.add_embed(embed))
        })
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    Ok(())
}

pub async fn poker_leave(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let guild_id = command
        .guild_id
        .ok_or(GenericError::new(&"Guild ID not found"))?;
    let cid = command.channel_id;
    let uid = command.user.id;

    if let Some(mut state) = session::load_state(guild_id, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
    {
        if !state.players.contains(&uid.0) {
            respond_ephemeral(command, ctx, "You are not in this poker game.").await?;
            return Ok(());
        }

        state.remove_player(uid);
        session::save_state_or_err(guild_id, cid, &state).await?;
    }

    respond_ephemeral(command, ctx, "You have left the poker game.").await?;
    Ok(())
}

fn get_integer_option(
    options: &[serenity::model::prelude::interaction::application_command::CommandDataOption],
    name: &str,
) -> Option<i64> {
    for opt in options {
        if opt.name == name {
            if let Some(CommandDataOptionValue::Integer(v)) = opt.resolved.as_ref() {
                return Some(*v);
            }
        }
    }
    None
}
