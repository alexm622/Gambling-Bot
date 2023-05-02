use serenity::model::{
    prelude::{ChannelId, GuildId},
    user::User,
};

use crate::{redis::poker::get_user_hand, sql::structs::poker_hand_to_emojis};

pub async fn get_poker_draw_embed(
    guild_id: GuildId,
    cid: ChannelId,
    user: User,
) -> Result<serenity::builder::CreateEmbed, String> {
    let mut embed = serenity::builder::CreateEmbed::default();
    embed.title("Poker Draw");

    //your hand

    let hand = get_user_hand(guild_id, cid, user.id)
        .await
        .expect("error getting user hand");

    let hand_str = poker_hand_to_emojis(hand);

    //add hand_str to embed
    embed.description(&hand_str);

    return Ok(embed);
}

pub async fn get_poker_hand(
    guild_id: GuildId,
    cid: ChannelId,
    user: User,
) -> Result<serenity::builder::CreateEmbed, String> {
    let mut embed = serenity::builder::CreateEmbed::default();
    embed.title("Poker Draw");

    //your hand

    let hand = get_user_hand(guild_id, cid, user.id)
        .await
        .expect("error getting user hand");

    let hand_str = poker_hand_to_emojis(hand);

    //add hand_str to embed
    embed.description(&hand_str);

    return Ok(embed);
}
