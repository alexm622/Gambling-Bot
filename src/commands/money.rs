//money stuff

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::Message,
    prelude::Context,
};
use tracing::log::warn;

use crate::redis::users::get_user_bal;

#[command]
pub async fn bal(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let uid = msg.author.id;
    let bal = match get_user_bal(uid) {
        Ok(v) => v,
        Err(e) => {
            warn!("issue getting balance of uid: {}", uid);
            warn!("{}", e);
            return Ok(());
        }
    };

    msg.reply(ctx, format!("your current balance is: {}", bal))
        .await?;

    Ok(())
}
