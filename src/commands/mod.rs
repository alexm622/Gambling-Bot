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
                    //(red or black) or (odd or even) or an integer (value)
                    .create_option(|option| {
                        option
                            .name("bet_type")
                            .description("The type of bet you want to make")
                            .kind(CommandOptionType::String)
                            .required(true)
                            .add_string_choice("red", "red")
                            .add_string_choice("black", "black")
                            .add_string_choice("odd", "odd")
                            .add_string_choice("even", "even")
                            .add_string_choice("integer", "integer")
                    })
                    //if bet_type is integer, this is required
                    .create_option(|option| {
                        option
                            .name("integer")
                            .description("The integer you want to bet on")
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
            .create_application_command(|command| {
                command
                    .name("poker")
                    .description("Play poker")
                    .create_option(|option| {
                        option
                            .name("bet")
                            .description("The amount you want to bet")
                            .kind(CommandOptionType::Integer)
                            .required(true)
                    })
            })
            .create_application_command(|command| {
                command
                    .name("blackjack")
                    .description("Play blackjack")
                    .create_option(|option| {
                        option
                            .name("bet")
                            .description("The amount you want to bet")
                            .kind(CommandOptionType::Integer)
                            .required(true)
                    })
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
