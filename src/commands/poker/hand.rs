use serenity::{
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType, MessageFlags,
    },
    prelude::Context,
};

use crate::{commands::poker::hand_evaluator, errors::GenericError};

use super::{respond_ephemeral, session};

pub async fn poker_hand_handler(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let guild_id = command
        .guild_id
        .ok_or(GenericError::new(&"Guild ID not found"))?;
    let cid = command.channel_id;
    let uid = command.user.id;

    let state = session::load_state_or_err(guild_id, cid).await?;

    if !state.players.contains(&uid.0) {
        respond_ephemeral(command, ctx, "You are not in this poker game.").await?;
        return Ok(());
    }

    let embed = get_hand_embed(&state, uid);

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| {
                    m.content("Your poker hand")
                        .add_embed(embed)
                        .flags(MessageFlags::EPHEMERAL)
                })
        })
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    Ok(())
}

fn get_hand_embed(
    state: &session::PokerGameState,
    uid: serenity::model::prelude::UserId,
) -> serenity::builder::CreateEmbed {
    let hole_cards = state.hole_cards_eval(uid);

    let hole_str = if hole_cards.is_empty() {
        "No hole cards yet.".to_string()
    } else {
        hand_evaluator::cards_to_discord_emojis(&hole_cards)
    };

    let mut embed = serenity::builder::CreateEmbed::default();
    embed.title("Your Hole Cards");
    embed.description(hole_str);
    embed.color(serenity::utils::Colour::DARK_GREEN);
    embed
}
