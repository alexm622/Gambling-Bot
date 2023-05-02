use crate::errors::GenericError;
use serenity::{
    model::prelude::{interaction::{application_command::ApplicationCommandInteraction, MessageFlags, InteractionResponseType}},
    prelude::Context,
};

use tracing::{trace, warn};

pub mod poker_draw;



pub async fn poker_command_handler(
    command: ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), GenericError> {
    let name = command.data.name.clone();

    trace!("poker command called: {}", name);
    trace!("balance called");
    let guild_id = command.clone().guild_id.unwrap();
    let user = command.clone().user;
    let cid = command.clone().channel_id;

    trace!("guild id: {:?}", guild_id);
    trace!("user: {:?}", user);


    match PokerCommandsEnum::from_str(&name){
        PokerCommandsEnum::Check => {
            //run the command
        }
        PokerCommandsEnum::Call => {
            //run the command
        }
        PokerCommandsEnum::Discard(v) => {
            //run the command
        }
        PokerCommandsEnum::Draw => {
            //run the command
            let embed = poker_draw::get_poker_draw_embed( guild_id, cid, user).await.expect("error getting poker draw embed");
            //respond with an ephemeral message
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message
                        .content("Poker Draw")
                        .add_embed(embed)
                        .flags(MessageFlags::EPHEMERAL)
                    )
            }).await.expect("error sending poker draw embed");
        }
        PokerCommandsEnum::Hand => {
            //run the command
            let embed = poker_draw::get_poker_hand( guild_id, cid, user).await.expect("error getting poker hand embed");
            //respond with an ephemeral message
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message
                        .content("Poker Draw")
                        .add_embed(embed)
                        .flags(MessageFlags::EPHEMERAL)
                    )
            }).await.expect("error sending poker draw embed");
        }
        PokerCommandsEnum::Fold => {
            //run the command
        }
        PokerCommandsEnum::Join => {
            //run the command
        }
        PokerCommandsEnum::Leave => {
            //run the command
        }
        PokerCommandsEnum::Start => {
            //run the command
        }
        PokerCommandsEnum::Raise(v) => {
            //run the command
        }
        PokerCommandsEnum::AllIn => {
            //run the command
        }
        PokerCommandsEnum::InvalidCommand => {
            //run the command
        }
    }



    Ok(())
}

pub enum PokerCommandsEnum {
    Join,
    Leave,
    Start,
    Draw,
    Hand,
    Discard(u8),
    Fold,
    Raise(u64),
    Check,
    Call,
    AllIn,
    InvalidCommand,
}

impl PokerCommandsEnum{
    pub fn from_str(command: &str) -> PokerCommandsEnum{
        match command{
            "pjoin" => PokerCommandsEnum::Join,
            "pstart" => PokerCommandsEnum::Start,
            "pleave" => PokerCommandsEnum::Leave,
            "pallin" => PokerCommandsEnum::AllIn, 
            "pdraw" => PokerCommandsEnum::Draw,
            "phand" => PokerCommandsEnum::Hand,
            "pdiscard" => PokerCommandsEnum::Discard(0),
            "pfold" => PokerCommandsEnum::Fold,
            "praise" => PokerCommandsEnum::Raise(0),
            "pcheck" => PokerCommandsEnum::Check,
            "pcall" => PokerCommandsEnum::Call,
            _ => PokerCommandsEnum::InvalidCommand,
        }
    }

    pub fn to_str(&self) -> &str{
        match self{
            PokerCommandsEnum::Join => "pjoin",
            PokerCommandsEnum::Leave => "pleave",
            PokerCommandsEnum::Start => "pstart",
            PokerCommandsEnum::Draw => "pdraw",
            PokerCommandsEnum::Hand => "phand",
            PokerCommandsEnum::Discard(_v) => "discard",
            PokerCommandsEnum::Fold => "pfold",
            PokerCommandsEnum::Raise(_v) => "praise",
            PokerCommandsEnum::AllIn => "pallin",
            PokerCommandsEnum::Check => "pcheck",
            PokerCommandsEnum::Call => "pcall",
            PokerCommandsEnum::InvalidCommand => "invalid",
        }
    }

    pub fn with_raise_amount(&self, amount: u64) -> PokerCommandsEnum{
        match self{
            PokerCommandsEnum::Raise(_v) => PokerCommandsEnum::Raise(amount),
            _ => PokerCommandsEnum::InvalidCommand,
        }
    }

    /**
     * This function is used to create a PokerCommandsEnum::Discard with the amount of cards to discard.
     * amount is a u8 because you can only discard up to 5 cards.
     * it is represented as a binary number, so 0b1000 would discard the first card, 0b0100 would discard the second card, etc.
     */
    pub fn with_discard_amount(&self, amount: u8) -> PokerCommandsEnum{
        match self{
            PokerCommandsEnum::Discard(_v) => PokerCommandsEnum::Discard(amount),
            _ => PokerCommandsEnum::InvalidCommand,
        }
    }
}

pub fn string_to_u8_bin(string: &str) -> u8{
    let mut result: u8 = 0;
    let mut i = 0;
    for c in string.chars(){
        if c == '1'{
            result += 2u8.pow(i);
        }
        i += 1;
    }
    result
}

pub fn u8_to_string_bin(num: u8) -> String{
    let mut result = String::new();
    let mut num = num;
    while num > 0{
        if num % 2 == 1{
            result.push('1');
        }else{
            result.push('0');
        }
        num /= 2;
    }
    result
}