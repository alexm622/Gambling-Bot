use serenity::prelude::SerenityError;
use serenity::{
    model::prelude::{
        command::{Command, CommandOptionType},
    },
    prelude::Context,
};



pub mod blackjack;
pub mod money;
pub mod poker;
pub mod roulette;
pub mod slots;

//create commands using GuildID::set_application_commands
pub async fn register_commands(
    ctx: &Context,
) -> Result<Vec<serenity::model::application::command::Command>, SerenityError> {
    let commands = Command::set_global_application_commands(&ctx.http, |commands| {
        commands
            .create_application_command(|command| {
                command.name("help").description("Get help with the bot")
            })
            //MONEY COMMANDS
            //balance
            .create_application_command(|command| {
                command
                    .name("bal")
                    .description("Get balance")
                    //optional command to get another user's balance
                    .create_option(|option| {
                        option
                            .name("user")
                            .description("The user you want to get the balance of")
                            .kind(CommandOptionType::User)
                            .required(false)
                    })
            })
            //reset self balance
            //this should have a confirmation message and a 5 minute timer
            .create_application_command(|command| {
                command.name("reset_bal").description("Reset your balance")
            })
            // 
            //ROULETTE COMMANDS
            //roulette bet
            .create_application_command(|command| {
                command
                    .name("roulette")
                    .description("Play roulette")
                    .create_option(|option| {
                        option
                            .name("bet")
                            .description("The amount you want to bet")
                            .kind(CommandOptionType::Integer)
                            .required(true)
                    })
                    //(red or black) or green or(odd or even) or (low or high) or an integer (value)
                    .create_option(|option| {
                        option
                            .name("bet_type")
                            .description("The type of bet you want to make")
                            .kind(CommandOptionType::String)
                            .required(true)
                            .add_string_choice("red", "red")
                            .add_string_choice("black", "black")
                            .add_string_choice("green", "green")
                            .add_string_choice("even", "even")
                            .add_string_choice("odd", "odd")
                            .add_string_choice("low", "low")
                            .add_string_choice("high", "high")
                            .add_string_choice("number", "number")
                    })
                    //if bet_type is integer, this is required
                    .create_option(|option| {
                        option
                            .name("number")
                            .description("The number you want to bet on")
                            .kind(CommandOptionType::Integer)
                            .required(false)
                    })
            })
            //roulette odds
            .create_application_command(|command| {
                command
                    .name("roulette_odds")
                    .description("Get the odds for roulette")
                    //create option string for bet (red or black) or (odd or even) or an integer
                    .create_option(|option| {
                        option
                            .name("bet")
                            .description("The bet that you want to get the odds for")
                            .kind(CommandOptionType::String)
                            .required(true)
                    })
            })
            //roulette table
            //view the bets on the table right now
            .create_application_command(|command| {
                command
                    .name("roulette_table")
                    .description("View the bets on the table")
            })
            //
            // POKER COMMANDS
            //poker draw
            .create_application_command(|command| {
                command
                    .name("pdraw")
                    .description("Draw cards for poker")
            })
            //poker fold
            .create_application_command(|command| {
                command
                    .name("pfold")
                    .description("Fold your hand for poker")
            })
            //poker hand
            .create_application_command(|command| {
                command
                    .name("phand")
                    .description("View your hand for poker")
            })
            //poker raise
            .create_application_command(|command| {
                command
                    .name("praise")
                    .description("Raise your bet for poker")
                    .create_option(|option| {
                        option
                            .name("bet")
                            .description("The amount you want to raise your bet by")
                            .kind(CommandOptionType::Integer)
                            .required(true)
                    })
            })
            //poker call
            .create_application_command(|command| {
                command
                    .name("pcall")
                    .description("Call the current bet for poker")
            })
            //poker check
            .create_application_command(|command| {
                command
                    .name("pcheck")
                    .description("Check the current bet for poker")
            })
            //poker all in
            .create_application_command(|command| {
                command
                    .name("pallin")
                    .description("Go all in for poker")
            })
            //poker discard
            .create_application_command(|command| {
                command
                    .name("pdiscard")
                    .description("Discard cards for poker")
                    .create_option(|option| {
                        option
                            .name("cards")
                            .description("The cards you want to discard")
                            .kind(CommandOptionType::String)
                            .required(true)
                    })
            })
            //poker start
            .create_application_command(|command| {
                command
                    .name("pstart")
                    .description("Start a game of poker")
            })
            //poker join
            .create_application_command(|command| {
                command
                    .name("pjoin")
                    .description("Join a game of poker")
            })
            //poker leave
            .create_application_command(|command| {
                command
                    .name("pleave")
                    .description("Leave a game of poker")
            })
            //MOD COMMANDS
            //reset user balance
            //this should have a confirmation message and a 5 minute timer
            //limit to only users with the mod role
            //can only effect users with a lower role than the mod
            //user has to be in the server
            .create_application_command(|command| {
                command
                    .name("reset_user_bal")
                    .description("Reset a user's balance")
                    .create_option(|option| {
                        option
                            .name("user")
                            .description("The user you want to reset the balance of")
                            .kind(CommandOptionType::User)
                            .required(true)
                    })
            })
    })
    .await;
    commands
}
