use serenity::{
    builder::CreateEmbed,
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType, MessageFlags,
    },
    prelude::Context,
};
use tracing::trace;

use crate::{
    commands::poker::{flow, session},
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

    flow::start_lobby(ctx, guild_id, cid, uid).await?;

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
        session::save_state(guild_id, cid, &state)
            .await
            .map_err(|e| GenericError::new(&e.to_string()))?;
    }

    poker::remove_user_from_joined(guild_id, cid, uid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    poker::remove_user_hand(guild_id, cid, uid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;
    poker::clear_user_bet(guild_id, cid, uid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    respond_ephemeral(command, ctx, "You have left the poker game.").await?;
    Ok(())
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
