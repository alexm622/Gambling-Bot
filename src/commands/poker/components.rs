use serenity::{
    model::prelude::{
        interaction::{
            message_component::MessageComponentInteraction, InteractionResponseType, MessageFlags,
        },
        ChannelId, GuildId, UserId,
    },
    prelude::Context,
};
use tracing::warn;

use crate::{
    commands::poker::hand_evaluator,
    errors::GenericError,
};

use super::{
    flow::{self, PlayerAction},
    session,
    ui,
};

pub async fn handle_poker_component(
    component: &MessageComponentInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let parts: Vec<&str> = component.data.custom_id.split(':').collect();
    if parts.len() < 4 || parts[0] != "poker" {
        return Ok(());
    }

    let action = parts[1];
    let gid = GuildId(
        parts[2]
            .parse()
            .map_err(|_| GenericError::new(&"Invalid guild id"))?,
    );
    let cid = ChannelId(
        parts[3]
            .parse()
            .map_err(|_| GenericError::new(&"Invalid channel id"))?,
    );
    let uid = component.user.id;

    // showcards needs to reply directly with an ephemeral message
    if action == "showcards" {
        if parts.len() < 5 {
            return Ok(());
        }
        let expected_uid = UserId(
            parts[4]
                .parse()
                .map_err(|_| GenericError::new(&"Invalid user id"))?,
        );
        if uid != expected_uid {
            return Ok(());
        }
        return handle_show_cards(component, ctx, gid, cid, uid).await;
    }

    // all other actions are acknowledged silently; results are broadcast in the channel
    component
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::DeferredUpdateMessage)
                .interaction_response_data(|m| m.flags(MessageFlags::EPHEMERAL))
        })
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    match action {
        "join" => {
            flow::handle_join(ctx, gid, cid, uid).await?;
        }
        "start" => {
            flow::start_game_now(ctx, gid, cid, uid).await?;
        }
        "fold" => {
            if parts.len() < 5 {
                return Ok(());
            }
            let expected_uid = UserId(
                parts[4]
                    .parse()
                    .map_err(|_| GenericError::new(&"Invalid user id"))?,
            );
            if uid != expected_uid {
                return Ok(());
            }
            flow::handle_action(ctx, gid, cid, uid, PlayerAction::Fold, None).await?;
        }
        "checkcall" => {
            if parts.len() < 5 {
                return Ok(());
            }
            let expected_uid = UserId(
                parts[4]
                    .parse()
                    .map_err(|_| GenericError::new(&"Invalid user id"))?,
            );
            if uid != expected_uid {
                return Ok(());
            }
            flow::handle_action(ctx, gid, cid, uid, PlayerAction::CheckCall, None).await?;
        }
        "raise" => {
            if parts.len() < 6 {
                return Ok(());
            }
            let expected_uid = UserId(
                parts[4]
                    .parse()
                    .map_err(|_| GenericError::new(&"Invalid user id"))?,
            );
            if uid != expected_uid {
                return Ok(());
            }
            let amount = parts[5]
                .parse::<u64>()
                .map_err(|_| GenericError::new(&"Invalid raise amount"))?;
            flow::handle_action(ctx, gid, cid, uid, PlayerAction::Raise, Some(amount)).await?;
        }
        "allin" => {
            if parts.len() < 5 {
                return Ok(());
            }
            let expected_uid = UserId(
                parts[4]
                    .parse()
                    .map_err(|_| GenericError::new(&"Invalid user id"))?,
            );
            if uid != expected_uid {
                return Ok(());
            }
            flow::handle_action(ctx, gid, cid, uid, PlayerAction::AllIn, None).await?;
        }
        _ => {
            warn!("unknown poker component action: {}", action);
        }
    }

    Ok(())
}

async fn handle_show_cards(
    component: &MessageComponentInteraction,
    ctx: &Context,
    gid: GuildId,
    cid: ChannelId,
    uid: UserId,
) -> Result<(), GenericError> {
    let state = session::load_state(gid, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
        .ok_or(GenericError::new(&"No poker game found."))?;

    let hole_cards = state
        .hole_cards
        .get(&uid.0)
        .cloned()
        .unwrap_or_default();

    if hole_cards.is_empty() {
        return Err(GenericError::new(&"No hole cards found for you."));
    }

    let hole_str = hole_cards
        .iter()
        .map(|&c| {
            let eval = hand_evaluator::card_tuple_to_eval(crate::utils::deck::int_to_card(c));
            hand_evaluator::card_to_emoji(eval)
        })
        .collect::<Vec<_>>()
        .join(" ");

    let embed = ui::create_hand_embed(&hole_str);

    component
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| m.set_embed(embed).flags(MessageFlags::EPHEMERAL))
        })
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?;

    Ok(())
}
