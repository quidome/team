# Technical spec: The message for the next game

**Status**: in progress
**Size**: story
**Change**: [2026-10-29-next-game-message](./)
**Date**: 2026-10-29

## Objective
The coordinator copies one Dutch message about the team's next game and the table and referee
places of the team in the coming week, from a part "Messages" on the start page, so the team and
the parents know when and where, and who drives or has table or referee duty.

## Requirements summary
Story C of the epic [Duties](../2026-10-18-duties/) (`functional.md` approved 2026-10-25, second
change, changed 2026-10-27; epic `technical.md` approved 2026-10-25, second change). Stories A and
B are built, closed and released (`v0.5.0`, decision 0023).

- **The start page** gets a part "Messages" below "Next" with the link "Message for the next game". It shows when the team has a coming game (today or later), or table or referee places of the team in the coming week. Without it, the part is not shown. ("Message for the referee committee" is story D.)
- **The message page** shows the Dutch text and "Copy". "Copy" puts the text on the clipboard and shows "Copied". When the browser refuses, the text stays shown and selectable, with "Select the text and copy it."
- **The text** (exact forms under step 2):
  - the team's next game: date and start time, "Thuis" or "Uit", the opponent, the hall and the town; "tijd nog niet bekend" without a start time;
  - for an away game: "Rijden:" with the drivers by first name, open places as "nog N open";
  - "Tafel en scheidsrechter deze week:": each game from today to the coming Sunday (today itself on a Sunday) with table or referee places of the team, whether or not the team plays that week; date, start time, the game, and the players by first name, open places as "nog N open";
  - two players with the same first name: "Anna B.", "Anna V." (`display_names`);
  - without a coming game of the team: only the week part; without places in the week: no week part.
- **Built from what is stored at that moment**; nothing about a message is stored. The coordinator may edit the text after pasting.
- **Edge cases**: no coming game and no places this week: no link. The next game is no longer in the game list: the message is for the next game that is. Copy refused: see above.

**Readings of the spec, confirmed by the coordinator (2026-10-29):**
- **R1.** The next game is the next game of the team in the game list, also when it is weeks away. A game today stays the next game all day, also after its start time.
- **R2.** The line "Rijden" shows only when the game needs drivers of the team and their number is above 0.
- **R3.** Only places that count are in the message: a coming game that left the game list, or that is no longer a home game of the club, is not in it (the rule of the totals, `SchedulePart::counts`).
- **R4.** The court is not in the message (the spec names the hall and the town).
- **R5.** At each game of the week part, "Tafel" comes before "Scheidsrechter"; a kind without a place of the team at that game has no line.
- The texts are in the core code, compiled in; templates the coordinator can edit are out of scope (`docs/ideas.md`, 2026-10-10).

## Current state
- **The duties of a game**: `team_core::game_places` → `GamePlaces { schedule: Option<SchedulePart>, drivers: Option<DriverPart> }`. `SchedulePart` has the 4 places, `of_team: Vec<DutyPlace>` (the places that name the team, with `filled`) and `counts`; `DriverPart` has `places: Vec<DutyPlace>` and `counts`. `DutyPlace { kind, number, filled }`.
- **The players on the places**: `team_core::Assignment { kind, number, planned, outcome }`; for a coming game the planned player is the one shown.
- **The names**: `team_core::display_names(players)` ("Anna", "Anna B."); the server keeps them in `SeasonDuties::names`.
- **The games of a season**: `api::season::games_of(state, season, current, today) -> SeasonGames { club, team, schedule: Schedule { coming, past }, duties: SeasonDuties, … }`; `schedule.coming` is in time order from today, each a `ScheduledGame { game, reason: TeamGame | DutyGame | Kept, in_list }`. `places_of_season(&games, today) -> PlacesOf` (the duties of each game by source id).
- **The start page**: `api/start.rs` `GET /api/start` already calls `games_of` and `places_of_season` for the current season; `StartAnswer { open_items, last, next, last_games, last_games_season }`; front end `web/src/routes/+page.svelte` and `web/src/lib/start.ts`.
- **The window of "Next"** (`start_page::next_until`) is the Sunday of **next** week; the coming week of the message is shorter (to the coming Sunday) and is a new rule.
- `Game { date, start_time: Option<NaiveTime>, home: Side, away: Side, place: Place { hall, town, court } }`; `Side { team, team_name, … }`.
- **Patterns**: rules in the core, worked out at each request (decisions 0020, 0022); one rule for a button and its page; routes under `/api` need a session (decision 0007); generated front-end types (`just api-types`); logs never contain player names.

## Steps

### Step 1: The clipboard (spike)
- **Goal**: know that "Copy" works and fails as the spec says, before the text is built (risk 4 of the epic).
- **Changes**: the page `web/src/routes/messages/next-game/+page.svelte` with a fixed text in a `<pre>` block (lines wrapped, `white-space: pre-wrap`) and "Copy": `try { await navigator.clipboard.writeText(text); copied = true } catch { failed = true }`. Without HTTPS `navigator.clipboard` does not exist; that error goes to the same `catch`. "Copied" stays until the page is left; on failure "Select the text and copy it." and the text stays. "Back" to the start page. Not linked from the start page yet. A new Playwright project `messages` (`messages.e2e.ts`) after `duties`, with the permissions `clipboard-read` and `clipboard-write`.
- **Tests** (browser): "Copy" shows "Copied" and the clipboard holds exactly the text on the page; with `writeText` made to fail by an init script, "Select the text and copy it." and the text stays. No phone test in this step (the coordinator, 2026-10-29); the phone is checked after step 4.

### Step 2: The text in the core
- **Goal**: the one rule for the message, with every text form fixed by an exact-text test.
- **Changes**: a new module `crates/core/src/messages.rs`, exported from `lib.rs`:
  ```rust
  /// A coming game of the season with its duties, as the messages read it.
  pub struct MessageGame<'a> {
      pub scheduled: &'a ScheduledGame,
      /// `game_places` of the game; `None`: no duties.
      pub places: Option<&'a GamePlaces>,
      pub assignments: &'a [Assignment],
  }

  /// The last day of the coming week: the coming Sunday; today on a Sunday.
  pub fn coming_week(today: NaiveDate) -> NaiveDate;

  /// The message for the next game. `games`: the coming games in time order; `names`:
  /// `display_names` of the season. `None`: no coming game of the team and no table or referee
  /// places of the team in the coming week.
  pub fn next_game_message(
      games: &[MessageGame<'_>],
      names: &BTreeMap<PlayerId, String>,
      team: TeamNumber,
      today: NaiveDate,
  ) -> Option<String>;
  ```
  - The next game: the first with `reason == TeamGame` and `in_list` (a `Kept` game is never the next game). "Thuis" when `game.home.team == team`, else "Uit"; the opponent is the other side's `team_name`.
  - Drivers: `places.drivers` when it `counts` and has places; the names of the filled places in the order of the place numbers, from the planned player of the assignment with the same kind and number.
  - The week part: the games with `today <= date <= coming_week(today)` whose `places.schedule` `counts` and has `of_team` places; per kind the filled names in place order, then the open ones.
  - A filled place whose player is not in `names` (not possible: the server loads every player on a place of the season) shows "?"; no panic.
  - Dutch day names (`maandag` … `zondag`; short `ma di wo do vr za zo`) and month names (`januari` … `december`; short `jan feb mrt apr mei jun jul aug sep okt nov dec`) as two small tables in the module.
- **The text forms** (one empty line between the parts; no empty line at the end):
  ```
  Volgende wedstrijd: zaterdag 31 oktober, 14:30
  Uit tegen B.Z. '72 M16 1
  Sporthal Bulwijk, Woerden

  Rijden: Anna, Bram, Cas (nog 1 open)

  Tafel en scheidsrechter deze week:
  za 31 okt 16:00 Blue Drakes X12 1 – De Dunckers X12 1
    Tafel: Eva, nog 1 open
    Scheidsrechter: Gijs
  ```
  - No start time: `Volgende wedstrijd: zaterdag 31 oktober, tijd nog niet bekend`; in the week part `za 31 okt, tijd nog niet bekend, Blue Drakes X12 1 – De Dunckers X12 1` (a form chosen in this design; the spec does not give it).
  - Home game of the team: `Thuis tegen …`, no line "Rijden".
  - Drivers: all filled `Rijden: Anna, Bram, Cas`; none filled `Rijden: nog 3 open`; 0 drivers or not needed: no line (R2).
  - Table and referee: all filled `  Tafel: Eva, Fleur`; none filled `  Tafel: nog 2 open` (a comma, as in the example; the driver line uses brackets).
  - Without a next game the text starts with `Tafel en scheidsrechter deze week:`. A game of the team with table or referee places of the team is in both parts.
  - The dash between the teams is `–` (U+2013), as in the spec.
- **Tests** (core, exact text):
  1. `coming_week`: Mon 26 Oct 2026 → Sun 1 Nov; Sat 24 Oct → Sun 25 Oct; Sun 25 Oct (the clock goes back) → 25 Oct; Sun 27 Dec → 27 Dec; Mon 28 Dec → Sun 3 Jan 2027.
  2. The full example above, character for character (away game Sat 31 Oct 14:30 with 4 drivers, 3 filled; a home game of another club team at 16:00 with table 1 Eva, table 2 open, referee 1 Gijs).
  3. A home game of the team: `Thuis tegen …`, no line "Rijden".
  4. No start time: the first line and the week line.
  5. Drivers all filled, none filled, 0 drivers.
  6. Table and referee all filled, none filled, only referee places (no line "Tafel"), "Tafel" before "Scheidsrechter".
  7. No next game, places this week: the text starts with the week part.
  8. A next game, no places this week: no week part, no empty line at the end.
  9. Nothing: `None`.
  10. Not counted (R3): a coming game no longer in the game list, a game no longer a home game of the club; a `Kept` game is never the next game.
  11. The window: a place on the next Monday is not in the week part; on a Sunday only that day.
  12. A game today is the next game, also with a start time before now (R1).
  13. Two players "Anna": `Anna B.` and `Anna V.`.
  14. The next game weeks away, nothing this week: it is still the next game.

### Step 3: The route and the start page flag
- **Goal**: the server gives the text and tells the start page whether to show the link, from one function.
- **Changes**:
  - `crates/server/src/api/messages.rs`:
    ```rust
    /// The one rule of the link on the start page and of the message page (decision 0022).
    pub(crate) fn next_game_text(games: &SeasonGames, places: &PlacesOf, today: NaiveDate) -> Option<String>;

    #[derive(Debug, Serialize, ToSchema)]
    pub struct MessageView { pub text: Option<String> }

    /// GET /api/messages/next-game -> 200 { "text": string | null }
    pub async fn next_game(State(state): State<AppState>) -> Result<Json<MessageView>, AppError>;
    ```
    It builds the `MessageGame`s from `games.schedule.coming`, `places` and `games.duties.assignments_of(id)`, and calls `next_game_message` with `games.duties.names` and the team. Without a chosen team: `None`.
  - `GET /api/messages/next-game` reads the current season with `games_of` and `places_of_season`, as the start page does. `null` is a normal answer, not an error (no 404).
  - `GET /api/start`: `StartAnswer` gets `messages: MessagesView { next_game: bool }`, set from `next_game_text(&games, &places, today).is_some()` with the data it already has.
  - `app.rs`, `api/mod.rs`: the route (with a session, decision 0007) and the OpenAPI; `just api-types` for `web/openapi.json` and `web/src/lib/api/schema.d.ts`.
  - Errors: only the existing database errors (`AppError`, 500). The route only reads; no lock. The text is never logged.
  - Measure `GET /api/start` once with the full test season (limit about 200 ms locally; about 30 ms after story B).
- **Tests** (server, `crates/server/tests/messages.rs`, and the lists of routes):
  1. Without a login, `GET /api/messages/next-game` gives 401; the route is in the lists of `tests/players.rs` and `tests/player_logs.rs`.
  2. Without a chosen team: `{ "text": null }`, and `messages.next_game` is false on the start page.
  3. With the test game list and the clock at 26 Oct 2026: the text starts with the team's next game, with a driver and a table player saved through the existing routes.
  4. The start page and the route agree: `messages.next_game` is true exactly when `text` is not `null` (with and without a message).
  5. No player name in the logs of the route (as `tests/player_name_logs.rs`).
  - The existing `tests/start.rs` still passes (with the new field).

### Step 4: The page and "Messages" on the start page
- **Goal**: the coordinator opens and copies the real message from the start page.
- **Changes**:
  - `web/src/lib/api/client.ts`: `loadNextGameMessage()`; the type of the start answer with `messages`.
  - `web/src/routes/messages/next-game/+page.ts` and `+page.svelte`: the text from the route; with `text: null` "No message: no coming game and no duties this week." and "Back".
  - `web/src/routes/+page.svelte`: the part "Messages" below "Next", with the link "Message for the next game" when `messages.next_game`; without it the part is not shown.
- **Tests** (browser, `messages.e2e.ts`): the start page shows "Messages" and "Message for the next game", a tap opens the page with the text; "Back" goes to the start page; a phone width (390 pixels) has no sideways scroll and no accessibility errors (the check the other pages use). The existing `start.e2e.ts` still passes. Then the coordinator taps "Copy" on the phone after the release.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| The message is made in the core (`messages`), in Dutch, from what is stored at the moment of the request; nothing about a message is stored | Rules in one place, exact-text tests (epic decision "messages in the core") | Yes, 0024 at Close |
| The start page gets a flag from the same function as the message route | The link and the message cannot differ (decision 0022); one request for the start page | Same record |
| The texts are compiled in, not templates | Templates the coordinator can edit are out of scope (coordinator, 2026-10-29) | Same record |
| `{ "text": null }` for no message, not 404 | No message is a normal state | No |
| A new struct `MessageGame` instead of extending `GameDuties` | The message needs the reason and `in_list` of a game; the totals do not | No |

## Not in scope
- The message for the referee committee and its link (story D).
- A departure time, the court, templates the coordinator can edit, other languages.
- A "Share" button (the share menu of the phone): the spec asks for "Copy".
- A next game in the next season (its games are not read before July).
- Any change to the places, the totals or the other rules of the start page.

## Test plan
- **Existing tests that must still pass**: core `duties` and `start_page`; `tests/start.rs` (new field `messages`); `tests/players.rs` and `tests/player_logs.rs` (the new route needs a login); `start.e2e.ts` and the other browser tests.
- **New tests**: per step, see above (core 1–14, server 1–5, browser in `messages.e2e.ts`).
- **Checks**: while working only the tests of the changed part; `just check` once before a commit.

## Compatibility and performance
- New route `GET /api/messages/next-game`; the answer of `GET /api/start` gets the field `messages` (the front-end types are generated).
- The start page shows the new part "Messages" when there is a message.
- `GET /api/start` does one more pass over the coming games it already has; measured in step 3.
- Player names in the text come decrypted from `players.rs` (decision 0021) and are never logged.

## Progress
- [x] Step 1: the clipboard (spike)
- [x] Step 2: the text in the core
- [x] Step 3: the route and the start page flag (`GET /api/start` median 8 ms, debug build, the test game list of the season without players)
- [x] Step 4: the page and "Messages" on the start page
- [x] All checks pass (`just check`; `api-types-check` passes after the commit of the new types)

## Deviations
None from `functional.md`. Readings that the design makes exact: R1–R5 (see Requirements summary);
the week line without a start time (`za 31 okt, tijd nog niet bekend, …`), which the spec does not
give.

## Review findings
