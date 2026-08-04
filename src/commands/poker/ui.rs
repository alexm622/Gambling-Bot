use serenity::{
    builder::{CreateActionRow, CreateButton, CreateEmbed},
    model::prelude::{component::ButtonStyle, ChannelId, GuildId, UserId},
};

use super::session::{Phase, PokerGameState};

pub fn create_lobby_embed(state: &PokerGameState, seconds_remaining: u64) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("Poker Game Lobby");
    embed.description(format!(
        "Join the game! Starting in **{}** seconds.\nPlayers: {}",
        seconds_remaining,
        state
            .players
            .iter()
            .map(|id| format!("<@{}>", id))
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

pub fn create_status_embed(state: &PokerGameState) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    let phase_name = match state.phase {
        Phase::Lobby => "Lobby",
        Phase::FirstBet => "First Betting Round",
        Phase::Draw => "Draw Phase",
        Phase::SecondBet => "Second Betting Round",
        Phase::Showdown => "Showdown",
        Phase::Finished => "Finished",
    };

    let current = state
        .current_player()
        .map(|u| format!("<@{}>", u.0))
        .unwrap_or_else(|| "None".to_string());

    let players = state
        .players
        .iter()
        .map(|id| {
            let folded = if state.folded.contains(id) {
                " (folded)"
            } else {
                ""
            };
            format!(
                "<@{}>{} - bet: {}",
                id,
                folded,
                state.player_bet(UserId::from(*id))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    embed.title(format!("Poker Game - {}", phase_name));
    embed.description(format!(
        "Pot: **{}**\nCurrent bet: **{}**\nCurrent turn: {}\n\n{}",
        state.pot, state.current_bet, current, players
    ));
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

    let mut row1 = CreateActionRow::default();
    row1.add_button(fold_button);
    row1.add_button(checkcall_button);

    let mut row2 = CreateActionRow::default();
    row2.add_button(raise50_button);
    row2.add_button(raise100_button);
    row2.add_button(raise200_button);

    let mut row3 = CreateActionRow::default();
    row3.add_button(allin_button);

    vec![row1, row2, row3]
}

pub fn create_draw_embed(hand_emoji: &str) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("Draw Phase");
    embed.description(format!(
        "Your hand:\n{}\n\nUse `/pdiscard` to replace cards (e.g. `1 3 5`).",
        hand_emoji
    ));
    embed.color(serenity::utils::Colour::BLUE);
    embed
}

pub fn create_showdown_embed(winners: &[(UserId, String)], pot: u64) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("Showdown");
    let desc = if winners.is_empty() {
        "No winners.".to_string()
    } else {
        format!(
            "Pot: **{}**\nWinner{}: {}",
            pot,
            if winners.len() > 1 { "s" } else { "" },
            winners
                .iter()
                .map(|(u, hand)| format!("<@{}> with {}", u.0, hand))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    embed.description(desc);
    embed.color(serenity::utils::Colour::GOLD);
    embed
}

pub fn create_timeout_embed(uid: UserId) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("Turn Timed Out");
    embed.description(format!("<@{}> took too long and folded.", uid.0));
    embed.color(serenity::utils::Colour::DARK_RED);
    embed
}
