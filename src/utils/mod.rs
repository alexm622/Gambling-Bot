pub mod card_ascii;
pub mod cleanup;
pub mod command_handler;
pub mod component_handler;
pub mod deck;
pub mod money;
pub mod roulette;

//convert the gameid in to a name
pub fn game_id_to_name(game_id: u8) -> String {
    match game_id {
        0 => String::from("poker"),
        _ => String::from("unknown"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_id_to_name_maps_known_games() {
        assert_eq!(game_id_to_name(0), "poker");
        assert_eq!(game_id_to_name(1), "unknown");
        assert_eq!(game_id_to_name(255), "unknown");
    }
}
