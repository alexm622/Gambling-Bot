use serenity::{
    model::prelude::{
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
            MessageFlags,
        },
    },
    prelude::Context,
};

use crate::{commands::poker::hand_evaluator, errors::GenericError};

use super::session;

pub async fn poker_draw_handler(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let guild_id = command
        .guild_id
        .ok_or(GenericError::new(&"Guild ID not found"))?;
    let cid = command.channel_id;
    let uid = command.user.id;

    let state = session::load_state(guild_id, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
        .ok_or(GenericError::new(&"No poker game found."))?;

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

pub async fn poker_hand_handler(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    poker_draw_handler(command, ctx).await
}

fn get_hand_embed(
    state: &session::PokerGameState,
    uid: serenity::model::prelude::UserId,
) -> serenity::builder::CreateEmbed {
    let hole_cards = state
        .hole_cards
        .get(&uid.0)
        .cloned()
        .unwrap_or_default();

    let hole_str = if hole_cards.is_empty() {
        "No hole cards yet.".to_string()
    } else {
        hole_cards
            .iter()
            .map(|&c| {
                let eval = hand_evaluator::card_tuple_to_eval(crate::utils::deck::int_to_card(c));
                hand_evaluator::card_to_emoji(eval)
            })
            .collect::<Vec<_>>()
            .join(" ")
    };

    let mut embed = serenity::builder::CreateEmbed::default();
    embed.title("Your Hole Cards");
    embed.description(hole_str);
    embed.color(serenity::utils::Colour::DARK_GREEN);
    embed
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
