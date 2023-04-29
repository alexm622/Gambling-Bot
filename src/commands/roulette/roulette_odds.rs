use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};

//run the command
pub fn get_odds_string(options: &[CommandDataOption]) -> String {
    let option = options
        .get(0)
        .expect("Expected bet")
        .resolved
        .as_ref()
        .expect("Expected bet value");
    //option is a string
    if let CommandDataOptionValue::String(s) = option {
        return get_odds(s);
    } else {
        return String::from("Expected option to be a string");
    }
}

//run, return embed
pub fn get_odds_embed(options: &[CommandDataOption]) -> serenity::builder::CreateEmbed {
    let option = options
        .get(0)
        .expect("Expected bet")
        .resolved
        .as_ref()
        .expect("Expected bet value");
    //option is a string
    if let CommandDataOptionValue::String(s) = option {
        return get_odds_embed_string(s);
    } else {
        return serenity::builder::CreateEmbed::default();
    }
}

//embeds
fn get_odds_embed_string(option: &str) -> serenity::builder::CreateEmbed {
    let mut embed = serenity::builder::CreateEmbed::default();
    let odds = get_odds(option);
    if odds == "Invalid option" {
        embed.title("Invalid option");
        embed.description("Valid options are: red, black, odd, even, integer");
    } else {
        embed.title(format!("{} odds", option));
        embed.description(odds);
    }
    embed
}

//strings

fn get_odds(option: &str) -> String {
    match option {
        "red" => String::from("1 to 1"),
        "black" => String::from("1 to 1"),
        "odd" => String::from("1 to 1"),
        "even" => String::from("1 to 1"),
        //try to parse as an integer
        _ => {
            if let Ok(i) = option.parse::<i32>() {
                if i >= 1 && i <= 36 {
                    return String::from("35:1");
                }
            }
            String::from("Invalid option")
        }
    }
}
