use serenity::model::{prelude::{UserId, GuildId, interaction::application_command::{CommandDataOptionValue, CommandDataOption}}, user::User};

pub async fn get_bal_embed(
    options: &[CommandDataOption],
    guild_id: GuildId,
    origin: User,
) -> Result<serenity::builder::CreateEmbed, String> {
    //see if there is a user specified
    
    let user: User;

    let option =  options.get(0);

    match option{
        Some(v) =>{
            match v.resolved.as_ref(){
                Some(v) => {
                    match v{
                        CommandDataOptionValue::User(u,_) => {
                            user = u.clone();
                        }
                        _ => {
                            return Err(String::from("Expected option to be a user"));
                        }
                    }
                }
                None => {
                    user = origin;
                }
            }
        }
        None => {
            user = origin;
        }
    }


    
    

    let bal = match get_bal(user.id, guild_id).await {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let mut embed = serenity::builder::CreateEmbed::default();
    embed.title(format!("{}'s balance", user.name));
    embed.description(format!("{}'s balance is: {}", user.name, bal));
    Ok(embed)
}

pub async fn get_bal(id: UserId, guild: GuildId) -> Result<i64, String> {
    let bal = crate::redis::users::get_user_bal(id, guild).await;
    match bal {
        Ok(v) => Ok(v),
        Err(e) => Err(e.to_string()),
    }
}