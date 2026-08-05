use serenity::{
    builder::{CreateActionRow, CreateButton, CreateEmbed},
    model::prelude::{component::ButtonStyle, ChannelId, GuildId, UserId},
};

use super::{
    bot::is_bot,
    hand_evaluator,
    session::{Phase, PokerGameState},
};

pub fn player_name(uid: UserId) -> String {
    if is_bot(uid) {
        "Bot".to_string()
    } else {
        format!("<@{}>", uid.0)
    }
}

pub fn format_community_cards(state: &PokerGameState) -> String {
    if state.community_cards.is_empty() {
        return "None".to_string();
    }

    let cards: Vec<_> = state
        .community_cards
        .iter()
        .map(|&c| hand_evaluator::card_tuple_to_eval(crate::utils::deck::int_to_card(c)))
        .collect();
    hand_evaluator::cards_to_discord_emojis(&cards)
}

pub fn create_lobby_embed(state: &PokerGameState, seconds_remaining: u64) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("Poker Game Lobby");
    embed.description(format!(
        "Join the game! Starting in **{}** seconds.\nPlayers: {}",
        seconds_remaining,
        state
            .players
            .iter()
            .map(|id| player_name(UserId::from(*id)))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    embed.color(serenity::utils::Colour::GOLD);
    embed
}

pub fn create_lobby_buttons(gid: GuildId, cid: ChannelId) -> Vec<CreateActionRow> {
    let mut join_button = CreateButton::default();
    join_button
        .label("Join")
        .style(ButtonStyle::Primary)
        .custom_id(format!("poker:join:{}:{}", gid, cid));

    let mut start_button = CreateButton::default();
    start_button
        .label("Start Now")
        .style(ButtonStyle::Success)
        .custom_id(format!("poker:start:{}:{}", gid, cid));

    let mut row = CreateActionRow::default();
    row.add_button(join_button);
    row.add_button(start_button);
    vec![row]
}

pub fn format_table_overview(state: &PokerGameState) -> String {
    let phase_name = match state.phase {
        Phase::Lobby => "Lobby",
        Phase::PreFlop => "Pre-Flop",
        Phase::Flop => "Flop",
        Phase::Turn => "Turn",
        Phase::River => "River",
        Phase::Showdown => "Showdown",
        Phase::Finished => "Finished",
    };

    let current = state
        .current_player()
        .map(player_name)
        .unwrap_or_else(|| "None".to_string());

    let players = state
        .players
        .iter()
        .map(|id| {
            let uid = UserId::from(*id);
            let folded = if state.folded.contains(id) {
                " (folded)"
            } else {
                ""
            };
            format!(
                "{} {} - bet: {}",
                player_name(uid),
                folded,
                state.player_bet(uid)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let community = format_community_cards(state);

    format!(
        "Phase: {}\nPot: **{}**\nCurrent bet: **{}**\nCurrent turn: {}\nCommunity cards:\n{}\n\nPlayers:\n{}",
        phase_name, state.pot, state.current_bet, current, community, players
    )
}

pub fn create_status_embed(state: &PokerGameState) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("Poker Table");
    embed.description(format_table_overview(state));
    embed.color(serenity::utils::Colour::DARK_GREEN);
    embed
}

pub fn create_action_buttons(gid: GuildId, cid: ChannelId, uid: UserId) -> Vec<CreateActionRow> {
    let mut fold_button = CreateButton::default();
    fold_button
        .label("Fold")
        .style(ButtonStyle::Danger)
        .custom_id(format!("poker:fold:{}:{}:{}", gid, cid, uid));

    let mut checkcall_button = CreateButton::default();
    checkcall_button
        .label("Check / Call")
        .style(ButtonStyle::Primary)
        .custom_id(format!("poker:checkcall:{}:{}:{}", gid, cid, uid));

    let mut raise50_button = CreateButton::default();
    raise50_button
        .label("Raise 50")
        .style(ButtonStyle::Secondary)
        .custom_id(format!("poker:raise:{}:{}:{}:50", gid, cid, uid));

    let mut raise100_button = CreateButton::default();
    raise100_button
        .label("Raise 100")
        .style(ButtonStyle::Secondary)
        .custom_id(format!("poker:raise:{}:{}:{}:100", gid, cid, uid));

    let mut raise200_button = CreateButton::default();
    raise200_button
        .label("Raise 200")
        .style(ButtonStyle::Secondary)
        .custom_id(format!("poker:raise:{}:{}:{}:200", gid, cid, uid));

    let mut allin_button = CreateButton::default();
    allin_button
        .label("All In")
        .style(ButtonStyle::Danger)
        .custom_id(format!("poker:allin:{}:{}:{}", gid, cid, uid));

    let mut show_cards_button = CreateButton::default();
    show_cards_button
        .label("Show Cards")
        .style(ButtonStyle::Secondary)
        .custom_id(format!("poker:showcards:{}:{}:{}", gid, cid, uid));

    let mut row1 = CreateActionRow::default();
    row1.add_button(fold_button);
    row1.add_button(checkcall_button);

    let mut row2 = CreateActionRow::default();
    row2.add_button(raise50_button);
    row2.add_button(raise100_button);
    row2.add_button(raise200_button);

    let mut row3 = CreateActionRow::default();
    row3.add_button(allin_button);
    row3.add_button(show_cards_button);

    vec![row1, row2, row3]
}

pub fn create_action_prompt_embed(state: &PokerGameState, uid: UserId) -> CreateEmbed {
    let mut prompt = CreateEmbed::default();
    prompt.title(format!("{}'s Turn", player_name(uid)));
    prompt.description(format!(
        "{}\n\nChoose your action below. Use **Show Cards** to view your hand privately.",
        format_table_overview(state)
    ));
    prompt.color(serenity::utils::Colour::GOLD);
    prompt
}

pub fn create_hand_embed(hand_emoji: &str) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("Your Hand");
    embed.description(hand_emoji);
    embed.color(serenity::utils::Colour::BLUE);
    embed
}

pub fn create_timeout_embed(uid: UserId) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("Turn Timed Out");
    embed.description(format!("{} took too long and folded.", player_name(uid)));
    embed.color(serenity::utils::Colour::DARK_RED);
    embed
}

pub fn create_action_result_embed(
    state: &PokerGameState,
    actor: UserId,
    action_text: &str,
) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("Player Action");
    embed.description(format!(
        "{} {}\n\n{}",
        player_name(actor),
        action_text,
        format_table_overview(state)
    ));
    embed.color(serenity::utils::Colour::DARK_GREEN);
    embed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::poker::session::BOT_USER_ID;

    #[test]
    fn player_name_distinguishes_bot() {
        assert_eq!(player_name(UserId::from(BOT_USER_ID)), "Bot");
        assert_eq!(player_name(UserId::from(123)), "<@123>");
    }

    #[test]
    fn community_cards_empty_shows_none() {
        let state = PokerGameState::new(UserId::from(1));
        assert_eq!(format_community_cards(&state), "None");
    }

    #[test]
    fn table_overview_shows_phase_pot_and_players() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.advance_phase(); // -> PreFlop
        state.place_bet(UserId::from(1), 50);

        let overview = format_table_overview(&state);
        assert!(overview.contains("Phase: Pre-Flop"), "{}", overview);
        assert!(overview.contains("Pot: **50**"), "{}", overview);
        assert!(overview.contains("<@1>"), "{}", overview);
        assert!(overview.contains("<@2>"), "{}", overview);
        assert!(!overview.contains("(folded)"), "{}", overview);
    }

    #[test]
    fn table_overview_marks_folded_players() {
        let mut state = PokerGameState::new(UserId::from(1));
        state.add_player(UserId::from(2));
        state.fold(UserId::from(2));

        let overview = format_table_overview(&state);
        assert!(overview.contains("<@2>  (folded)"), "{}", overview);
    }
}
