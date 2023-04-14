pub mod card_ascii;
pub mod cleanup;
pub mod deck;
pub mod poker;
pub mod roulette;

pub const SIZE_POKER: u8 = 52;

//convert the gameid in to a name
pub fn game_id_to_name(game_id: u8) -> String {
    match game_id {
        0 => String::from("poker"),
        _ => String::from("unknown"),
    }
}
