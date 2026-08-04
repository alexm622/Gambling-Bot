use serenity::{
    model::prelude::interaction::{
        message_component::MessageComponentInteraction, InteractionResponseType,
    },
    prelude::Context,
};
use tracing::{info, warn};

use crate::{commands::poker::components::handle_poker_component, errors::GenericError};

pub async fn component_handler(
    component: MessageComponentInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    info!("component handler called");

    let _data = component.data.clone();
    let _uid = component.user.id;
    let _cid = component.channel_id;
    let _gid = component.guild_id;
    let _mid = component.message.id;

    // route poker buttons by custom_id prefix
    if component.data.custom_id.starts_with("poker:") {
        return handle_poker_component(&component, ctx).await;
    }

    warn!("unhandled component: {}", component.data.custom_id);

    component
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content("Unknown button"))
        })
        .await
        .expect("error sending component clicked message");

    Ok(())
}
