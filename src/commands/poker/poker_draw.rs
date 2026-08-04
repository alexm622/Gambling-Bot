use serenity::{
    model::prelude::{
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
            MessageFlags,
        },
        ChannelId, GuildId,
    },
    prelude::Context,
};

use crate::{errors::GenericError, redis::poker, sql::structs::poker_hand_to_emojis};

pub async fn poker_draw_handler(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let guild_id = command
        .guild_id
        .ok_or(GenericError::new(&"Guild ID not found"))?;
    let cid = command.channel_id;
    let uid = command.user.id;

    if !poker::is_joinned(uid, guild_id, cid)
        .await
        .map_err(|e| GenericError::new(&e.to_string()))?
    {
        respond_ephemeral(command, ctx, "You are not in this poker game.").await?;
        return Ok(());
    }

    let embed = get_hand_embed(guild_id, cid, uid)
        .await
        .map_err(|e| GenericError::new(&e))?;

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

async fn get_hand_embed(
    guild_id: GuildId,
    cid: ChannelId,
    uid: serenity::model::prelude::UserId,
) -> Result<serenity::builder::CreateEmbed, String> {
    let hand = poker::get_user_hand(guild_id, cid, uid)
        .await
        .map_err(|e| e.to_string())?;

    let mut embed = serenity::builder::CreateEmbed::default();
    embed.title("Poker Hand");
    embed.description(poker_hand_to_emojis(hand));
    embed.color(serenity::utils::Colour::DARK_GREEN);
    Ok(embed)
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
