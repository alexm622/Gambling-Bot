use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};

//run, return embed
pub fn get_odds_embed(options: &[CommandDataOption]) -> serenity::builder::CreateEmbed {
    let option = options
        .first()
        .expect("Expected bet")
        .resolved
        .as_ref()
        .expect("Expected bet value");
    //option is a string
    if let CommandDataOptionValue::String(s) = option {
        get_odds_embed_string(s)
    } else {
        serenity::builder::CreateEmbed::default()
    }
}

//run the command
pub fn get_odds_string(options: &[CommandDataOption]) -> String {
    let option = options
        .first()
        .expect("Expected bet")
        .resolved
        .as_ref()
        .expect("Expected bet value");
    //option is a string
    if let CommandDataOptionValue::String(s) = option {
        get_odds(s)
    } else {
        String::from("Expected option to be a string")
    }
}

//embeds
fn get_odds_embed_string(option: &str) -> serenity::builder::CreateEmbed {
    let mut embed = serenity::builder::CreateEmbed::default();
    let odds = get_odds(option);
    if odds == "Invalid option" {
        embed.title("Invalid option");
        embed.description(
            "Valid options are: red, black, green, odd, even, low, high, or a number (0-36)",
        );
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
        "low" => String::from("1 to 1"),
        "high" => String::from("1 to 1"),
        // green covers 0 and 00: 2 of 38 numbers
        "green" => String::from("17 to 1"),
        //try to parse as an integer
        _ => {
            if let Ok(i) = option.parse::<i32>() {
                if (0..=36).contains(&i) {
                    return String::from("35 to 1");
                }
            }
            String::from("Invalid option")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn even_money_bets_pay_1_to_1() {
        for opt in ["red", "black", "odd", "even", "low", "high"] {
            assert_eq!(get_odds(opt), "1 to 1");
        }
    }

    #[test]
    fn green_pays_17_to_1() {
        assert_eq!(get_odds("green"), "17 to 1");
    }

    #[test]
    fn single_numbers_pay_35_to_1() {
        for n in 0..=36 {
            assert_eq!(get_odds(&n.to_string()), "35 to 1");
        }
    }

    #[test]
    fn invalid_options_rejected() {
        for opt in ["37", "-1", "abc", ""] {
            assert_eq!(
                get_odds(opt),
                "Invalid option",
                "expected {} to be invalid",
                opt
            );
        }
    }
}
