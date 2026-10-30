# Technical spec: The message of the week

**Status**: approved
**Size**: story
**Change**: [2026-10-30-message-of-the-week](./)
**Date**: 2026-10-30

## Objective
Change the built message for the next game into the message of the week, so the coordinator copies
one Dutch text about this week (Monday to Sunday) and puts the polls under it.

## Requirements summary
Story C of the epic [Duties](../2026-10-18-duties/) (`functional.md`, change of 2026-10-30).

- **The start page**: the part "Messages" has "Message of the week" when this week has a game of the team today or later, or table or referee places of the team. Otherwise the part is not shown. (Story D adds "Committee message".)
- **The page** keeps the text, "Copy", "Copied", "Select the text and copy it." and "Back". New title; without a message: "No message this week."
- **The text** (exact forms under step 1):
  - the heading with the week;
  - a block for each game of the team from today to Sunday, in time order: "Wedstrijd:", date and start time, "Thuis" or "Uit", the opponent, hall and town; "Rijden:" for an away game that needs drivers of the team and has more than 0 driver places;
  - "Tafel en scheidsrechter:" with the games of the **whole** week with table or referee places of the team, also days that have passed: "Scheidsrechter:" before "Tafel:", first names, open places as "nog N open";
  - a past place shows the player who did it, or "niemand";
  - only places that count; places of other club teams are never mentioned.
- **Built from what is stored at that moment**; nothing about a message is stored.

**Readings of the spec, made exact here:**
- **R1.** The heading always has both full dates: "Week 26 oktober – 1 november", "Week 12 oktober – 18 oktober", "Week 28 december – 3 januari".
- **R2.** A game of the team today stays in the message all day, also after its start time (as built).
- **R3.** A game of the team before today, also this week, gives no block, and does not make the button show. Its table or referee places stay in the second part.
- **R4.** A past place with "Nobody" shows "niemand" in place order: "Scheidsrechter: Gijs, niemand". It is not an open place.
- **R5.** A week that crosses into a new season (around 1 July) shows only the games of the current season, as all messages do. Rare, outside the playing season.
- **R6.** A game without a start time keeps the built form: `za 31 okt, tijd nog niet bekend, Blue Drakes X12 1 – De Dunckers X12 1`.
- **R7.** "Thuis" stays the built rule: a game in the club's own hall (decision 0024).

## Current state
- `crates/core/src/messages.rs`: `MessageGame { scheduled, places, assignments }`, `coming_week(today)`, `next_game_message(games, names, club, team, today)`, the private `long_date`, `short_date`, `clock`, `filled_and_open(places, assignments, names, brackets)`, `next_game_part`, `week_game`. "Tafel" comes first and only the planned player is read.
- `team_core::who_did(assignment, past) -> Counted { Done, Planned, Nobody }` (story B): the one rule for who counts.
- `crates/server/src/api/messages.rs`: `MessageView { text }`, `next_game_text(games, places, today)` reads only `schedule.coming`; `GET /api/messages/next-game`.
- `api/season.rs`: `schedule.past` is newest date first, with time order within a date; `places_of_season` already covers past and coming games, so past places need no new query.
- `api/start.rs`: `MessagesView { next_game }` from the same function.
- Front end: `web/src/routes/messages/next-game/`, `web/src/lib/api/client.ts` (line 512), the part "Messages" in `web/src/routes/+page.svelte`, generated `web/openapi.json` and `schema.d.ts`.
- Tests: core `messages` (17 exact-text tests), `tests/messages.rs`, `tests/message_logs.rs`, the route lists in `tests/players.rs` and `tests/player_logs.rs`, `web/e2e/messages.e2e.ts` (clock Monday 12 October 2026).

## Steps

### Step 1: The text in the core
- **Goal**: the one rule of the message of the week, every form fixed by an exact-text test.
- **Changes** in `crates/core/src/messages.rs`, exported from `lib.rs` (`coming_week` and `next_game_message` go away):
  ```rust
  /// Monday and Sunday of the week that holds `date`.
  pub fn week_of(date: NaiveDate) -> (NaiveDate, NaiveDate);

  /// The message of the week. `games`: the games of the season with their duties; within a date
  /// in time order, the dates in any order. `None`: no game of the team from today to Sunday and
  /// no table or referee places of the team in the week.
  pub fn week_message(
      games: &[MessageGame<'_>],
      names: &BTreeMap<PlayerId, String>,
      club: ClubNumber,
      team: TeamNumber,
      today: NaiveDate,
  ) -> Option<String>;
  ```
  - `week_message` filters on the week, then sorts with a stable sort on the date.
  - A block: `reason == TeamGame`, `in_list`, `today <= date <= sunday`. `next_game_part` becomes `game_block`, with "Wedstrijd:" and "Rijden:" in the same block.
  - `filled_and_open` loses `brackets` and gets `past: bool`. It reads each filled place through `who_did`: `Done` and `Planned` give the name, `Nobody` gives "niemand", a name not known gives "?".
  - `week_game` becomes `duty_lines(game, names, today)`, with "Scheidsrechter" first and `past = date < today`. It takes the names as an argument, so stories D (full names) and E use it unchanged. It stays private until they need it.
- **The text forms** (one empty line between parts; no empty line at the end):
  ```
  Week 26 oktober – 1 november

  Wedstrijd: zaterdag 31 oktober, 14:30
  Uit tegen B.Z. '72 M16 1
  Sporthal Bulwijk, Woerden
  Rijden: Anna, Bram, Cas, nog 1 open

  Tafel en scheidsrechter:
  za 31 okt 16:00 Blue Drakes X12 1 – De Dunckers X12 1
    Scheidsrechter: Gijs
    Tafel: Eva, nog 1 open
  ```
  Start time not known: `Wedstrijd: zaterdag 31 oktober, tijd nog niet bekend`.
- **Tests** (core, exact text):
  1. `week_of`: Mon 26 Oct 2026 → 26 Oct, 1 Nov; Sun 25 Oct (the clock goes back) → 19 Oct, 25 Oct; Sat 2 Jan 2027 → 28 Dec, 3 Jan.
  2. The example above, character for character.
  3. Heading forms of R1.
  4. Two games of the team: two blocks, one empty line between them, in time order.
  5. Home game: "Thuis", no "Rijden"; a game against another club team in the club's hall is "Thuis".
  6. Away game with 0 drivers: no "Rijden".
  7. Drivers all filled, none filled ("Rijden: nog 3 open").
  8. Table and referee: all filled, none filled, one kind only; "Scheidsrechter" before "Tafel"; places of another club team do not show.
  9. Only places: heading and second part.
  10. Only a game: no second part, no empty line at the end.
  11. Nothing: `None`. Only a game before today and no places: `None` (R3).
  12. On Friday: a game of Tuesday gives no block, its places stay, in date order before Saturday (R3).
  13. A game today after its start time stays (R2).
  14. Past place: "Someone else" shows the other player, "Nobody" shows "niemand", in place order (R4).
  15. Sunday: the whole week's places are in it; Monday of the next week is not.
  16. Not counted: a coming game no longer in the game list, or no longer a home game of the club; a `Kept` game gives no block.
  17. No start time: the block and the line of the second part (R6).
  18. Two players "Anna": "Anna B.", "Anna V.".

### Step 2: The route and the start page flag
- **Goal**: the server gives the text and tells the start page, from one function.
- **Changes**:
  - `api/messages.rs`: `week_text(games, places, today)` replaces `next_game_text`. It chains `schedule.past` (only dates in the week) and `schedule.coming`, maps them to `MessageGame`s and calls `week_message`. `GET /api/messages/week` (operation `weekMessage`) replaces the old route; the same `{ "text": string | null }`.
  - `api/start.rs`: `MessagesView { week: bool }`.
  - `app.rs`, `api/mod.rs`, and `just api-types` for `web/openapi.json` and `schema.d.ts`.
  - The route only reads; errors stay `AppError`; the text is never logged.
- **Tests** (`tests/messages.rs`, `tests/message_logs.rs`, route lists in `tests/players.rs`, `tests/player_logs.rs`):
  1. Without a login: 401. Without a team: `{ "text": null }` and `messages.week` false.
  2. Exact text from stored data, with the clock on Monday 26 October 2026: a game and a table place with a player.
  3. The clock on a day after a place in the same week: the place stays, with a correction "Someone else" showing the other player.
  4. A week with nothing: `null` and `messages.week` false; a week with something: true.
  5. The log has no player name of the message.

### Step 3: The page and the button
- **Goal**: the coordinator reaches the message from the start page.
- **Changes**:
  - Move `web/src/routes/messages/next-game/` to `web/src/routes/messages/week/`; title and heading "Message of the week"; empty text "No message this week."
  - `client.ts`: the route. `+page.svelte`: `start.messages.week`, the link "Message of the week" to `/messages/week`.
- **Tests** (`web/e2e/messages.e2e.ts`):
  1. The start page opens the message; the text on the page is the text of the server and starts with "Week "; "Back" returns.
  2. "Copy" and the refused clipboard: as now, on the new page.
  3. Phone width and accessibility: as now.
  4. Without a message: no part "Messages", and the page says "No message this week."

  The clock of the browser tests is Monday 12 October 2026. If the test data has nothing in that week, the first test adds a place in step 3.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| One function `duty_lines` with the names as an argument | Stories D and E need the same lines, with other names | No; decision 0024 is updated at the close |
| Past places read through `who_did` | One rule of who counts (decisions 0022, 0023) | No |
| The old route, page and flag are replaced, with no alias | Nothing of them is released | No |

## Not in scope
- The committee message (story D) and the duty overviews (story E). The button of the committee message comes with story D.
- Another week than this one; a departure time; message templates.

## Test plan
- **Existing tests that must still pass**: all others; the tests named under "Current state" are rewritten in their step.
- **New tests**: per step, above.

## Compatibility and performance
- Nothing of the old message is released, so no compatibility is needed.
- `GET /api/start` reads no new data: the past games and their places are already loaded.

## Progress
- [ ] Step 1
- [ ] Step 2
- [ ] Step 3
- [ ] All checks pass

## Deviations
None from `functional.md`.

## Review findings
