//help command

use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::log::trace;

#[command]
pub async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    trace!("Replying to message in {}", msg.channel_id);
    msg.reply(ctx, "no help for you").await?;

    Ok(())
}
