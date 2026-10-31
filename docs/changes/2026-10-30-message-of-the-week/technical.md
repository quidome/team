# Technical spec: The message of the week

**Status**: done
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
- [x] Step 1
- [x] Step 2
- [x] Step 3
- [x] All checks pass

## Deviations
None from `functional.md`.

From this spec:
- Steps 1 and 2 can be merged only together: step 1 removes `next_game_message`, so `crates/server` does not compile until step 2. No workaround; only the core is checked after step 1.
- Step 1 has 21 tests, not 18: the design's cases split into more tests.
- Step 2 changes the generated front-end types, so the front end does not pass its checks until step 3 (`client.ts` and the start page still name the old route and flag). Steps 1 to 3 belong in one commit.
- Step 3: the browser tests fix the answers of `GET /api/start` and `GET /api/messages/week` with a route in the test. The design planned a real table place, but the test data has no game in the week of the test clock (12 to 18 October 2026), and other browser tests depend on that. The text from stored data and the agreement of button and message stay tested in `crates/server/tests/messages.rs`, with no browser. Four of the five browser tests therefore do not run the real route (R2).

## Review findings

Build review, 2026-10-31. Input: the diff against `main` (all uncommitted), this folder, story C in the epic's `functional.md`, `docs/architecture.md`, decisions 0022, 0023, 0024, `CLAUDE.md`.

No finding breaks a decision or a rule. The text is in the core (0001, 0024), the button and the page use one function (0022), and past places go through `who_did` (0023).

- **R1. Medium: a past place with no player reads "nog N open", and alone it makes the button show.** `filled_and_open` counts open places without looking at `past` (`crates/core/src/messages.rs:110`). On Friday the message says "di 27 okt … Tafel: nog 1 open" about a game that was played, which reads as a request for help. Such a place also makes `week_message` return `Some`, so a week with nothing to announce still gets a button. This matters most while the coordinator catches up on past places (S19). The spec does not cover it: "open places as nog N open", and "a past place shows the player who did it … or niemand", which R4 settles only for "Nobody". The test `on_a_sunday_the_whole_week_is_in_it_and_not_the_next_monday` has exactly this place but asserts only the date, so the behaviour is not fixed by any test. Needs a decision by the coordinator, then a test: leave it out, "niemand", or keep it. Checked by trace, not by a run.
  - 2026-10-31: Fixed, the coordinator chose: a past place that stayed open shows "niemand", in place order; alone it does not make a message or the button. `functional.md` (stories C and D) says so; test `a_past_place_that_stayed_open_shows_niemand`.
- **R2. Medium: no browser test reaches the real route any more.** All five tests in `web/e2e/messages.e2e.ts` fix `GET /api/messages/week` and the flag of `GET /api/start`. The reason given under Deviations is true for four of them: the test data has no game from 12 to 18 October 2026 (checked in `games-37-2026-2027.json`: nearest dates 11 and 25 October). But that same fact means the real server already answers "no message" at the test clock. So the last test, "without a message", needs no fixed answers, and without them it would run the real route, the real flag and the real page end to end. Checked by reading the tests and a search of `web/e2e` for `messages`; the change itself was not run. The server tests do cover the text and the agreement of button and message (run, 5 passed).
  - 2026-10-31: Fixed: the test "without a message" runs the real routes; the four tests with a message keep fixed answers (see Deviations).
- **R3. Low: a comment the diff made false.** `crates/server/tests/message_logs.rs:1` still says "The message for the next game". Search: `next game|next-game|next_game|nextGame|coming_week` over `crates`, `web/src`, `web/e2e` and `web/openapi.json`. This was the only stale hit about the message; the hit in `crates/core/src/duties.rs:733` is about something else.
  - 2026-10-31: Fixed: the comment names the message of the week.
- **R4. Low: the week is filtered twice.** `week_text` drops past games before Monday (`crates/server/src/api/messages.rs`), and `week_message` filters on the same week again. Removing the server filter would change no result, so no test can fail on it. The design asked for it ("only dates in the week"). It is harmless, but it is a second place for the rule; the core alone is enough, since stories D and E call the same core. Checked by trace.
  - 2026-10-31: Not fixed: harmless, and the design asked for the server filter; it keeps old past games out of the list.

**Claims under Progress and Deviations:**
- "All checks pass": true. `just check` ran on this tree, exit 0: 122 browser tests, 281 core tests, clippy, svelte-check, `check-sqlx`.
- "Step 1 has 21 tests": true (count of `#[test]` in `messages.rs`, and a run).
- The test data has no game in the week of the test clock: true (see R2).

**Traced, no finding:**
- A game today is in `schedule.coming` and `past` is false, and `check_outcome` refuses a correction before the game, so a game today always shows the planned player.
- A coming gone or swapped game has `counts` false and gives no places and no block; a past gone game stays through `kept_games`.
- `names` holds every player on a place, also hidden ones (`load_duties`), so "?" cannot appear from a hidden player.
- `GET /api/start` passes the games of the current season to `week_text`, also in July and August, the same as the route.

**Stale on purpose, for `/close`:** `docs/product/messages.md`, `docs/product/start-page.md`, `docs/architecture.md` (lines 16, 19, 115) and decision 0024 still describe the message for the next game. The decision table above plans this for the close.

Outside this change: `.direnv/` and `.envrc` are untracked and not ignored, so a `git add -A` would pick them up.

Accept review, 2026-10-31, by a fresh agent. **Verdict: accepted with notes.** No concerning deviation. A1 and A2 are small choices for the coordinator, not corrections.

Input: story C and the facts of 2026-10-30 in the epic's `functional.md`, its `idea.md`, journey 4 in `docs/vision.md`, `change.md`, the diff against `main`. The view below was formed before reading Deviations. The arguments held only the mode and the change folder; nothing was ignored. There is no `idea.md` or `functional.md` in this folder: the epic's are the spec, as `change.md` says.

Run on this tree: core `messages` 22 passed; `tests/messages.rs` 5 and `tests/message_logs.rs` 1 passed; `just api-types-check` and `just check-web` passed; `npx playwright test messages` with the projects it depends on, 120 passed (the 5 of `messages.e2e.ts` among them). The program was not run with a filled week in a browser (see A4).

| Criterion of story C | Result | How checked |
|---|---|---|
| The part "Messages" below "Next", shown only with something to announce; tap, text, "Copy", "Copied" | met; A1 | diff of `+page.svelte`; browser tests (the positive case with fixed answers) |
| Heading; a block for each game from today to Sunday, in time order; "Thuis"/"Uit", opponent, hall, town; "Rijden:" only above 0 places | met | core tests with exact text, `tests/messages.rs` run; order traced through `schedule.past` and `within_date` (a stable sort keeps time order within a date) |
| "Tafel en scheidsrechter:" with "Scheidsrechter:" first, first names, "nog N open", no other club teams | met; A2 | core tests run |
| Past places stay until Sunday, played games leave, who did it or "niemand", a past place that stayed open does not alone show the button | met; A3 | core tests, `tests/messages.rs` (Sunday 1 November) run |
| "tijd nog niet bekend"; "Anna B.", "Anna V." | met | core tests run |
| Built from what is stored at that moment | met | trace: `week_text` at each request, nothing stored |
| Errors: "No message this week."; coming gone or swapped game left out, past game keeps what is stored; copying refused | met | browser test with the real routes; core tests; browser test |
| Example of the spec | met, character for character | `the_example_of_the_spec` run |

- **A1. Neutral drift: "Message of the week" is an underlined link, not a button.** Journey 4 and story C say "button"; `web/src/routes/+page.svelte:201` has the same text link as the links of the start page, while `Button` exists and is used on that page. It works and the tap is the same, and `technical.md` step 3 planned "the link". On a phone in the weekly routine a small link is the weaker target; the coordinator can say whether it matters. Checked: read the diff and search for `button` in `docs/vision.md`; not looked at on a phone.
  - 2026-10-31: Fixed, the coordinator chose: a real button on the start page.
- **A2. Neutral drift: no empty line between games in "Tafel en scheidsrechter:".** The spec shows only one game in story C, so it does not decide this. The games follow each other directly (`a_played_game_leaves_and_its_places_stay`), while the example of story D has an empty line between games and story E has one game. The two messages may differ; the choice is best made when story D reuses `duty_lines`, since the join is in `week_message`. Checked: read the code and the examples.
  - 2026-10-31: Fixed, the coordinator chose: one empty line between the games; `functional.md` says so.
- **A3. Neutral drift: a message made only of the past is possible.** "Messages announce what comes", but a week whose only table or referee place was filled and has passed still shows the button and a message with nothing to come, for example on a Sunday evening (`tests/messages.rs` shows this text on Sunday 1 November; a place with "Nobody" gives the same with "niemand"). The spec asks for this: places stay until Sunday, and only a place that stayed open was taken out (R1). It follows S25 and needs no change; `docs/ideas.md` line 2026-10-29 is the place for real use. The fact of 2026-10-30 "after a correction it shows the player who did it … or niemand" does not yet name a place that stayed open; story C does, so the stories are right and the fact is a line behind. Checked: code, run of the server test, read the facts.
  - 2026-10-31: Fixed: the fact of 2026-10-30 in `functional.md` names a place that stayed open.
- **A4. Faithful adaptation: the positive path in the browser uses fixed answers.** The reason under Deviations holds. Real text is tested through the API, the real page only with "no message". So no test shows a real text on the real page, and "Copy" on the phone is not tested. The wiring is one typed call and the page did not change beyond its name, so the risk is small; the line in `docs/backlog.md` covers the phone after the release. Checked: read `messages.e2e.ts`, run.
- **A5. Faithful adaptation: R5, the week across 1 July.** Only the games of the current season show. No games are expected around that date; not checked against the game list of a season change.
- **A6. Faithful adaptation: the spec was changed during the build (R1 of the build review).** "None from `functional.md`" under Deviations is true only because the coordinator's decision went into stories C and D. That is the way of working (rule 4), and the change is what a coordinator would mean: on Friday a played game does not ask for help.
