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

use crate::errors::GenericError;

use super::flow::{self, PlayerAction};

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

    // acknowledge the interaction
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
