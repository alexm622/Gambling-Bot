# TODO

## Phase 1: Foundation
- [ ] Shared SQL connection pool
  - Create one `Pool` in `sql::init_sql` and reuse it in all SQL modules
  - Replace per-call `Pool::new(...)` in `select.rs`, `insert.rs`, `delete.rs`
- [ ] SQL read layer
  - Convert `select.rs` to parameterized queries
  - Add typed helpers for stats, transactions, and leaderboards
- [ ] Transaction logging
  - Add `transactions` table: user_id, guild_id, amount, type, game, timestamp
  - Log every balance change (bet, win, loss, trade, daily)
- [ ] Startup refund
  - Refund all open roulette bets on boot
  - Clear stale poker/blackjack states and refund players
- [ ] Money commands
  - register only these commands in the guild id listed in secrets.csv under `admin_server`
  - blacklist the commands only be usable by userid listed in secrets.csv under `admin_list` (this will be a comma-separated list of user ids)
  - Implement `reset_bal` with confirmation + 5-minute timer
  - Implement `reset_user_bal` with mod role and lower-rank checks

## Phase 2: Roulette
- [ ] Single `/roulette` command with subcommands
  - `/roulette open`
  - `/roulette bet`
  - `/roulette close`
  - `/roulette table`
  - `/roulette odds`
- [ ] One table per guild (keyed by `guild_id`)
- [ ] Anyone can open a table
- [ ] Auto-close after 60 seconds and spin automatically
- [ ] Opener can force close with `/roulette close`
- [ ] Embed animation before final result
- [ ] Update SQL to group bets by `guild_id`

## Phase 3: Blackjack
- [ ] Single mode: `/blackjack bet:<amount>` vs dealer
- [ ] Multiplayer mode: `/blackjack lobby bet:<amount>`
  - 60-second lobby timer
  - Host can force-start
  - Each player bets their own amount
- [ ] Game actions: Hit, Stand, Double, Split, Surrender, Insurance
- [ ] Payouts: blackjack 3:2, win 1:1, push return bet, insurance 2:1, surrender half
- [ ] Redis state for single and multiplayer games
- [ ] Unit tests for hand scoring, dealer behavior, and payouts

## Phase 4: Slots
- [ ] `/slots bet:<amount>` with weighted reels
- [ ] Payout tiers: small, medium, big, jackpot
- [ ] Spin animation via message edits
- [ ] Unit tests for reel generation and payout calculation

## Phase 5: Profile, Stats, and Leaderboards
- [ ] `/profile [user]` command
  - Balance, total wagered, net profit, favorite game, streak
- [ ] `/leaderboard [balance|wins|biggest]` top 10
- [ ] SQL tables for `user_stats` and `transactions`
- [ ] Redis cache for leaderboards with TTL
- [ ] Achievement system: first win, big win, streak, poker showdown, blackjack 21

## Phase 6: Daily Rewards
- [ ] `/daily` command with 24-hour cooldown
- [ ] Streak bonus for consecutive days
- [ ] `/weekly` optional bonus
- [ ] Redis cooldown key + SQL log

## Phase 7: Chip Trading
- [ ] `/give <user> <amount>` direct transfer
- [ ] `/trade <user> <amount>` request with confirmation button
- [ ] Daily trade cap based on balance:
  - <10k: 10% of balance
  - <100k: 5% of balance
  - <1M: 2.5% of balance
  - >=1M: 1% of balance, capped at 1M
- [ ] 5% trade tax on new users
- [ ] SQL logging for all trades
- [ ] Prevent self-trade and bot-trade

## Phase 8: High Stakes Betting
- [ ] Balance-based tiers: 100k, 1M, 10M, 100M, ...
- [ ] Bet limits scale 10x per tier
- [ ] High-stakes leaderboard
- [ ] Big-win announcements

## Phase 9: Vanity
- [ ] `/title` commands (set, list, clear)
- [ ] Badges earned from achievements
- [ ] Profile embed color
- [ ] Full achievement list

## Order of Work
1. Phase 1: Foundation
2. Phase 2: Roulette
3. Phase 3: Blackjack (single first, then multiplayer lobby)
4. Phase 4: Slots
5. Phase 5: Profile/Stats/Leaderboards
6. Phase 6: Daily Rewards
7. Phase 7: Chip Trading
8. Phase 8: High Stakes
9. Phase 9: Vanity
