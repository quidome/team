# Technical spec: The message for the next game

**Status**: done
**Size**: story
**Change**: [2026-10-29-next-game-message](./)
**Date**: 2026-10-29

## Objective
The coordinator copies one Dutch message about the team's next game and the table and referee
places of the team in the coming week, from a part "Messages" on the start page, so the team and
the parents know when and where, and who drives or has table or referee duty.

## Requirements summary
Story C of the epic [Duties](../../2026-10-18-duties/) (`functional.md` approved 2026-10-25, second
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

### Build review (2026-10-29, fresh agent, diff `main...next-game-message` at `ea037d4`)
Checked by running: `cargo test -p team_core messages` (14 passed), `cargo test -p team --test messages --test start --test player_logs` (3 + 10 + 1 passed), `npm run build` and `npx playwright test --project=messages` with its dependency projects (120 passed), `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` (clean). No finding blocks Close.

What holds (traced): the rule is in the core (`messages::next_game_message`), the server only maps `SeasonGames` and `PlacesOf` into `MessageGame`s (decision 0001, "rules in the core"); the start page flag and the route call the same `next_game_text` with the data `GET /api/start` already reads (decision 0022), and a server test checks they agree with and without a message; the route is on the protected router and in the login lists (decision 0007); "counts" comes from the existing `game_places` (R3), not a new copy of the rule; the names come from `SeasonDuties::names` (`display_names`), nothing is logged by the route; no `unwrap`/`expect` outside tests (`coming_week` uses `checked_add_days(..).unwrap_or(today)`); the API types are generated; the page follows the load pattern of `/duties`.

- **R1. "Thuis"/"Uit" is decided by the home side, not by the hall, against `docs/language.md`.** `next_game_part` says "Thuis" only when `game.home.team == team`. The word list defines a home game as "a game of the team in the club's own hall". A game of the team as the away side against another team of the club (two club teams in one competition) is in the club's hall, needs no drivers (`required_duties`: `drivers` needs `home.club != club`), but the message says "Uit tegen Blue Drakes M16 2". The test data has no such game (checked: a script over `tests/data/game-list/games-37-2025-2026.json` and `games-37-2026-2027.json`, 0 games with `thuis_club_id == uit_club_id`), so it is rare; but the rule is cheap to make right (`game.home.club == club`, the same test as `required_duties`), and needs the club in `next_game_message` or in `MessageGame`. Checked: trace of `next_game_part` and `required_duties`; not run. Suggest: decide with the coordinator, add a core test either way.
  - 2026-10-29: Fixed (the coordinator's choice: the club's own hall): `next_game_message` takes the club, and "Thuis" means `game.home.club == club`, as `required_duties`; the opponent is still the other side. Core test `a_game_against_another_club_team_in_the_clubs_hall_is_at_home`.
- **R2. The log test of the new route cannot fail.** `tests/player_logs.rs` now calls `GET /api/messages/next-game`, but its `Coordinator::start` has no chosen team, so the answer is `{ "text": null }` and no player name is ever in the text (checked: read the test setup; `next_game_text` returns `None` without a team). Step 3, test 5 ("no player name in the logs of the route") is so only a check that the route answers 200. The route has no log line of its own (checked: search for `tracing::`/`info!` in `api/messages.rs`, none), so the risk is low; a test with a team and a player on a place (as in `tests/messages.rs`) with `LogBuffer` would make the claim true.
  - 2026-10-29: Fixed: a new test file `tests/message_logs.rs` (its own process, for `LogBuffer`) with M16 1 chosen and "Zephyrine Quistorp" on a table place of the team; it checks that the text holds "Tafel: Zephyrine", that the request is in the log, and that the name is not. The call in `tests/player_logs.rs` stays as a check that the route answers.
- **R3. Step 3, test 3 says "with a driver and a table player saved"; the test saves only a table player.** The next game of M16 1 on 31 October is a home game, so there is no line "Rijden" to fill (checked: read `tests/messages.rs`). The driver line is covered only in the core tests. Fix the line in the step, or add a driver at an away game in the server test.
  - 2026-10-29: Fixed: server test `an_away_game_has_its_drivers` (Monday 2 November 2026, the away game of 7 November, 2 drivers, one filled: "Rijden: Bram (nog 1 open)").
- **R4. Small: the core filters `date >= today` for the next game, which `schedule.coming` already guarantees** (checked: `schedule::schedule` puts a game of today or later in `coming`, test `a_game_today_is_coming_and_a_game_yesterday_is_past`). Harmless and makes the core function safe for any input; no change needed. Noted so story D does not copy it as a needed rule.
  - 2026-10-29: Not changed (the coordinator's choice): the check keeps the core function right for any input; it is not a rule that story D needs.
- **R5. Docs for Close** (not made false by the diff, but to update then): `docs/architecture.md` (core `messages`, `api/messages`, `StartAnswer.messages`, the route `/messages/next-game` in `web/`, the Playwright project `messages` after `duties` in "Patterns to follow"), `docs/product/start-page.md` (the part "Messages") and a product doc for the message; decision record 0024 as planned. The comment at the top of `web/src/routes/+page.svelte` was updated (checked: diff).
  - 2026-10-29: For Close.

### Accept review (2026-10-29, fresh agent, `main...next-game-message` at `befc510`)
Read: the epic's `functional.md` (context, facts, story C, Interface, Not in scope, spec review S23) and `idea.md`, `change.md`, this file's objective, the diff (core `messages.rs`, `api/messages.rs`, `api/start.rs`, the start page, the message page, the tests), and only then **Deviations**. Ran: `cargo test -p team_core messages` (15 passed), `cargo test -p team --test messages --test message_logs` (passed). The browser tests were not run again (the build review ran them: 120 passed). The arguments held only the mode and the change folder; nothing was ignored.

**Verdict: accepted with notes.** What was built is what was meant: one Dutch text, made at each visit from what is stored, that tells the team and the parents when and where, who drives, and who has table or referee duty this week, with open places as "nog N open" (S23). The differences are faithful adaptations or neutral drift; none is concerning. "None" under **Deviations** holds; the change of R1 of the build review moves the build towards `docs/language.md`, not away from the spec.

- **A1. Criterion 1, "Messages" below "Next", the link, "Copy" and "Copied": matches.** The part sits after the section "Next" and before the links; the link opens `/messages/next-game`; "Copy" writes the text and shows "Copied"; the text is in a `<pre>` that wraps and selects all on a tap. One difference: without a message the whole part "Messages" is hidden, where the spec only says the button is not shown. With one link that is the same to the coordinator. **Neutral drift**; story D must show the part when either message has something to say. Checked: read `web/src/routes/+page.svelte` (lines 162–211) and `messages.e2e.ts`.
- **A2. Criterion 2, the content: matches.** The example of the spec comes out character for character (core test `the_example_of_the_spec`), and the real game list gives the same form (server tests with a table player at a home game of X12 1 and a driver at the away game of 7 November). "Thuis" follows the club's own hall (`home.club == club`), as the word list defines a home game: **faithful adaptation** (R1 of the build review, the coordinator's choice). The court is left out (R4): **faithful**, the spec names the hall and the town. "Tafel" before "Scheidsrechter", a kind without a place of the team has no line (R5): **faithful**. The week part holds the games whether or not the team plays, and the next game is shown even when it is weeks away (R1 of the design): **faithful**. Checked: trace of `next_game_part`, `week_game`, `filled_and_open`; the runs above.
- **A3. Criterion 3, "tijd nog niet bekend": matches.** In the week part the form `za 31 okt, tijd nog niet bekend, …` was chosen by the design; the spec gives none and it reads as Dutch the parents understand. **Neutral drift.** Checked: core test `a_start_time_not_known_yet`.
- **A4. Criterion 4, built from what is stored: matches.** Nothing about a message is stored; the page asks the server at each visit, and the start page link comes from the same function (`next_game_text`), so the link and the text cannot disagree (server helper `message_of` checks it in every server test). Checked: read `api/messages.rs`, `api/start.rs`, `+page.ts`.
- **A5. Edge cases: match.** No coming game and no places this week: no link (`without_a_team_there_is_no_message`, `nothing_coming_gives_no_message`, the mocked e2e test). Copy refused: the text stays with "Select the text and copy it." (e2e with a failing `writeText`). The next game no longer in the game list: the filter `in_list` skips it, so the next game that is in the list is used. That branch is right by trace, but no test pins it: `places_that_do_not_count_are_left_out` tests a `Kept` game of the team (in the list) and a gone duty game, not a game of the team with `in_list == false`, which `schedule` does produce (`schedule.rs` test at line 374). A small gap in the tests, not in the behaviour. Checked: trace of `next_game_message`; search `gone()` and `in_list` in `messages.rs`.
  - 2026-10-29: Fixed: core test `a_next_game_no_longer_in_the_game_list_gives_way_to_the_next_one_that_is`.
- **A6. Points the spec left open, settled with the coordinator (R1–R3 of the design): faithful.** A game today stays the next game all day (test `a_game_today_is_the_next_game_all_day`); no line "Rijden" with 0 drivers or none needed; only places that count (the rule of the totals, `SchedulePart::counts`, not a new copy). All three match the intent "what the team needs to know now".
- **A7. Not checked yet: "Copy" on the coordinator's phone.** The clipboard is checked only in desktop Chromium (Playwright with clipboard permissions). The phone is the place where the message is copied into the chat; step 4 plans that tap after the release. Until then criterion 1 is met in the browser tests only. Not concerning (the call is made directly in the tap, which mobile browsers require, and the fallback covers a refusal), but the tap on the phone should be done and noted before or at Close. Checked: read step 4 and `+page.svelte`; not run on a phone.
  - 2026-10-29: Open (the coordinator's choice): the coordinator taps "Copy" on the phone after the release; a line in `docs/backlog.md` until then.
- **A8. Small, for use: on a Sunday the week part is that Sunday only.** A message made on Sunday evening shows the coming Saturday game but none of the next week's table and referee places. That is the spec's own rule ("today itself when it is a Sunday", test `the_week_part_ends_on_the_coming_sunday`) and fits "around training" on weekdays, so it is **faithful**; if the coordinator often makes the message on a Sunday, that is a line for `docs/ideas.md`, not a correction. Checked: trace of `coming_week`.
  - 2026-10-29: To `docs/ideas.md` (the coordinator's choice).
