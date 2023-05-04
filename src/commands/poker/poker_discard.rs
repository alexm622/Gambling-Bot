use serenity::{
    model::{
        prelude::{
            interaction::{application_command::ApplicationCommandInteraction, InteractionResponseType, MessageFlags}, ChannelId, GuildId, component::ButtonStyle,
        },
        user::User,
    },
    prelude::Context, builder::{CreateButton, CreateEmbed, },
};

use crate::{
    errors::GenericError, redis::poker::{get_user_hand,}, sql::structs::{PokerHand},
};

pub async fn poker_discard_handler(
    command: ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let guild_id = command.clone().guild_id.unwrap();
    let user = command.clone().user;
    let cid = command.clone().channel_id;

    let embed:CreateEmbed = get_poker_discard_embed_initial(guild_id, cid, user)
        .await
        .expect("error getting poker discard embed");

    //create buttons 1-5

    let buttons = create_buttons();

    //create button row

    let mut button_row = serenity::builder::CreateActionRow::default();

    for button in buttons {
        button_row.add_button(button);
    }


    //send message

    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| {
                m.flags(MessageFlags::EPHEMERAL)
                    .add_embed(embed)
                    .components(|c| c.add_action_row(button_row))
            })
    }).await.expect("error sending poker discard embed");
    
    Ok(())

}

async fn get_poker_discard_embed_initial(
    guild_id: GuildId,
    cid: ChannelId,
    user: User,
) -> Result<serenity::builder::CreateEmbed, String> {
    let mut embed = serenity::builder::CreateEmbed::default();
    embed.title("Poker Discard");

    //your hand

    let hand: PokerHand = get_user_hand(guild_id, cid, user.id)
        .await
        .expect("error getting user hand");

    //create a string matching the emojis of the hand with the numbers 1-5
    let emojis = hand.emoji_vec();

    let mut hand_str = String::new();
    
    for (i, emoji) in emojis.iter().enumerate() {
        hand_str.push_str(&format!("{}:\n {}\n{}\n", i+1, emoji.0,emoji.1));
    }

    let embed_description = format!("Your hand: \n {}", hand_str);

    //add hand_str to embed
    embed.description(&embed_description);

    return Ok(embed);
}

fn create_buttons() -> Vec<CreateButton>{
    let mut buttons: Vec<CreateButton> = Vec::new();

    //create buttons 1-5
    for i in 1..6 {
        let mut button = CreateButton::default();
        button.label(&i.to_string())
            .style(ButtonStyle::Primary)
            .custom_id(i);
        buttons.push(button);
    }

    return buttons;
}


