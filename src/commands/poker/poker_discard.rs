use serenity::{
    model::prelude::interaction::{
        application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
        InteractionResponseType, MessageFlags,
    },
    prelude::Context,
};
use tracing::trace;

use crate::{errors::GenericError, redis::poker, sql::structs::poker_hand_to_emojis};

pub async fn poker_discard_handler(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let guild_id = command
        .guild_id
        .ok_or(GenericError::new(&"Guild ID not found"))?;
    let cid = command.channel_id;
    let uid = command.user.id;

    trace!("poker discard command called by {}", uid);

    if !poker::is_joinned(uid, guild_id, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
    {
        respond_ephemeral(command, ctx, "You are not in this poker game.").await?;
        return Ok(());
    }

    let cards = match get_cards_option(&command.data.options) {
        Some(v) => v,
        None => {
            respond_ephemeral(
                command,
                ctx,
                "Please provide the cards you want to discard (e.g. `1 3 5`).",
            )
            .await?;
            return Ok(());
        }
    };

    // normalize to a string of unique digits 1-5, e.g. "135"
    let normalized = normalize_discard_input(&cards);

    if normalized.is_empty() {
        respond_ephemeral(
            command,
            ctx,
            "No valid cards selected. Use digits 1-5 (e.g. `1 3 5`).",
        )
        .await?;
        return Ok(());
    }

    let mut hand = poker::get_user_hand(guild_id, cid, uid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    hand.discard(normalized, uid, guild_id, cid)
        .await
        .map_err(|e| GenericError::new(&e))?;

    poker::push_poker_hand(hand, guild_id, cid, uid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    let embed = serenity::builder::CreateEmbed::default()
        .title("Poker Discard")
        .description(format!("Your new hand:\n{}", poker_hand_to_emojis(hand)))
        .color(serenity::utils::Colour::DARK_GREEN)
        .to_owned();

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| {
                    m.content("Cards discarded!")
                        .add_embed(embed)
                        .flags(MessageFlags::EPHEMERAL)
                })
        })
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    Ok(())
}

fn get_cards_option(
    options: &[serenity::model::prelude::interaction::application_command::CommandDataOption],
) -> Option<String> {
    for opt in options {
        if opt.name == "cards" {
            if let Some(CommandDataOptionValue::String(s)) = opt.resolved.as_ref() {
                return Some(s.clone());
            }
        }
    }
    None
}

fn normalize_discard_input(input: &str) -> String {
    let mut seen = [false; 5];
    let mut result = String::new();
    for c in input.chars() {
        if let Some(d) = c.to_digit(10) {
            if (1..=5).contains(&d) && !seen[d as usize - 1] {
                seen[d as usize - 1] = true;
                result.push(c);
            }
        }
    }
    result
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
