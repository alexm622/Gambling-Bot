use serenity::{model::prelude::interaction::{message_component::MessageComponentInteraction, InteractionResponseType}, prelude::Context};
use tracing::info;

use crate::errors::GenericError;

pub async fn component_handler(component: MessageComponentInteraction, ctx: &Context) -> Result<(), GenericError>{

    info!("component handler called");

    info!("component: {:?}", component);

    //basics: uid, cid, gid, mid, data

    let data = component.data.clone();
    let uid = component.user.id;
    let cid = component.channel_id;
    let gid = component.guild_id;
    let mid = component.message.id;



    //reply with a message saying the component was clicked

    component.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| message
                .content("component clicked")
            )
    }).await.expect("error sending component clicked message");

    Ok(())
}