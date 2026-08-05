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

use crate::{commands::poker::hand_evaluator, errors::GenericError};

use super::{
    flow::{self, PlayerAction},
    session, ui,
};

/// A parsed poker button custom_id.
///
/// Layout: `poker:<action>:<gid>:<cid>[:<expected_uid>[:<amount>]]`
/// where `expected_uid` is the player the button was rendered for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PokerComponent {
    Join {
        gid: GuildId,
        cid: ChannelId,
    },
    Start {
        gid: GuildId,
        cid: ChannelId,
    },
    ShowCards {
        gid: GuildId,
        cid: ChannelId,
        uid: UserId,
    },
    Action {
        gid: GuildId,
        cid: ChannelId,
        uid: UserId,
        action: PlayerAction,
        amount: Option<u64>,
    },
}

/// Parse a poker custom_id. Returns Ok(None) for ids that are not poker
/// components or are structurally incomplete, Err for malformed values.
pub fn parse_poker_custom_id(custom_id: &str) -> Result<Option<PokerComponent>, GenericError> {
    let parts: Vec<&str> = custom_id.split(':').collect();
    if parts.len() < 4 || parts[0] != "poker" {
        return Ok(None);
    }

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

    let parse_uid = |parts: &[&str]| -> Result<UserId, GenericError> {
        Ok(UserId(
            parts[4]
                .parse()
                .map_err(|_| GenericError::new(&"Invalid user id"))?,
        ))
    };

    match parts[1] {
        "join" => Ok(Some(PokerComponent::Join { gid, cid })),
        "start" => Ok(Some(PokerComponent::Start { gid, cid })),
        "showcards" => {
            if parts.len() < 5 {
                return Ok(None);
            }
            Ok(Some(PokerComponent::ShowCards {
                gid,
                cid,
                uid: parse_uid(&parts)?,
            }))
        }
        action_str => {
            let action = match PlayerAction::from_name(action_str) {
                Some(a) => a,
                None => {
                    warn!("unknown poker component action: {}", action_str);
                    return Ok(None);
                }
            };
            if parts.len() < 5 {
                return Ok(None);
            }
            let uid = parse_uid(&parts)?;
            let amount = if action == PlayerAction::Raise {
                if parts.len() < 6 {
                    return Ok(None);
                }
                Some(
                    parts[5]
                        .parse::<u64>()
                        .map_err(|_| GenericError::new(&"Invalid raise amount"))?,
                )
            } else {
                None
            };
            Ok(Some(PokerComponent::Action {
                gid,
                cid,
                uid,
                action,
                amount,
            }))
        }
    }
}

pub async fn handle_poker_component(
    component: &MessageComponentInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let parsed = match parse_poker_custom_id(&component.data.custom_id)? {
        Some(p) => p,
        None => return Ok(()),
    };

    let uid = component.user.id;

    // showcards needs to reply directly with an ephemeral message
    if let PokerComponent::ShowCards {
        gid,
        cid,
        uid: expected_uid,
    } = parsed
    {
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

    match parsed {
        PokerComponent::Join { gid, cid } => {
            flow::handle_join(ctx, gid, cid, uid).await?;
        }
        PokerComponent::Start { gid, cid } => {
            flow::start_game_now(ctx, gid, cid, uid).await?;
        }
        PokerComponent::Action {
            gid,
            cid,
            uid: expected_uid,
            action,
            amount,
        } => {
            if uid != expected_uid {
                return Ok(());
            }
            flow::handle_action(ctx, gid, cid, uid, action, amount).await?;
        }
        PokerComponent::ShowCards { .. } => unreachable!("handled above"),
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
    let state = session::load_state_or_err(gid, cid).await?;

    let hole_cards_eval = state.hole_cards_eval(uid);

    if hole_cards_eval.is_empty() {
        return Err(GenericError::new(&"No hole cards found for you."));
    }

    let hole_str = hand_evaluator::cards_to_discord_emojis(&hole_cards_eval);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_join_and_start() {
        assert_eq!(
            parse_poker_custom_id("poker:join:111:222").unwrap(),
            Some(PokerComponent::Join {
                gid: GuildId(111),
                cid: ChannelId(222),
            })
        );
        assert_eq!(
            parse_poker_custom_id("poker:start:111:222").unwrap(),
            Some(PokerComponent::Start {
                gid: GuildId(111),
                cid: ChannelId(222),
            })
        );
    }

    #[test]
    fn parses_showcards_with_uid() {
        assert_eq!(
            parse_poker_custom_id("poker:showcards:1:2:3").unwrap(),
            Some(PokerComponent::ShowCards {
                gid: GuildId(1),
                cid: ChannelId(2),
                uid: UserId(3),
            })
        );
    }

    #[test]
    fn parses_actions() {
        assert_eq!(
            parse_poker_custom_id("poker:fold:1:2:3").unwrap(),
            Some(PokerComponent::Action {
                gid: GuildId(1),
                cid: ChannelId(2),
                uid: UserId(3),
                action: PlayerAction::Fold,
                amount: None,
            })
        );
        assert_eq!(
            parse_poker_custom_id("poker:checkcall:1:2:3").unwrap(),
            Some(PokerComponent::Action {
                gid: GuildId(1),
                cid: ChannelId(2),
                uid: UserId(3),
                action: PlayerAction::CheckCall,
                amount: None,
            })
        );
        assert_eq!(
            parse_poker_custom_id("poker:allin:1:2:3").unwrap(),
            Some(PokerComponent::Action {
                gid: GuildId(1),
                cid: ChannelId(2),
                uid: UserId(3),
                action: PlayerAction::AllIn,
                amount: None,
            })
        );
        assert_eq!(
            parse_poker_custom_id("poker:raise:1:2:3:200").unwrap(),
            Some(PokerComponent::Action {
                gid: GuildId(1),
                cid: ChannelId(2),
                uid: UserId(3),
                action: PlayerAction::Raise,
                amount: Some(200),
            })
        );
    }

    #[test]
    fn rejects_non_poker_and_incomplete_ids() {
        assert_eq!(parse_poker_custom_id("other:join:1:2").unwrap(), None);
        assert_eq!(parse_poker_custom_id("poker:join:1").unwrap(), None);
        assert_eq!(parse_poker_custom_id("poker:fold:1:2").unwrap(), None);
        assert_eq!(parse_poker_custom_id("poker:raise:1:2:3").unwrap(), None);
        assert_eq!(parse_poker_custom_id("poker:bogus:1:2:3").unwrap(), None);
    }

    #[test]
    fn errors_on_malformed_ids() {
        assert!(parse_poker_custom_id("poker:join:abc:2").is_err());
        assert!(parse_poker_custom_id("poker:join:1:abc").is_err());
        assert!(parse_poker_custom_id("poker:fold:1:2:abc").is_err());
        assert!(parse_poker_custom_id("poker:raise:1:2:3:abc").is_err());
    }
}
