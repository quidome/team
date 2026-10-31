# Technical spec: The duty overviews

**Status**: done
**Size**: story
**Change**: [2026-10-31-duty-overviews](./)
**Date**: 2026-10-31

## Objective
Two Dutch texts to copy from the page "Duties", the driving overview and the table and referee
overview, with all coming duties of the current season, so the team and the parents see who does
what.

## Requirements summary
From story E of `../../2026-10-18-duties/functional.md`:

- "Duties" has the buttons "Driving overview" and "Table and referee overview" for the current season, not for an earlier season. Each opens a page with the text, "Copy" (as in the messages) and "Back" to "Duties".
- The driving overview: each coming away game of the team (today and later) that needs drivers of the team and has more than 0 driver places, in time order: short date, start time, "Uit tegen" the opponent and the town, then the drivers by first name and the open places.
- The table and referee overview: each coming game (today and later) with table or referee places of the team, in time order, in the form of the part "Tafel en scheidsrechter:" of the message of the week.
- No counts and no totals.
- Without coming duties of that kind: "Geen ritten gepland." or "Geen tafel- of scheidsrechtersdiensten gepland.", and "Copy" still works.
- What is in an overview follows the rule of the totals: a coming game no longer in the game list, or no longer needing duties of the team, is not in it.
- A start time not known yet shows "tijd nog niet bekend". A failed copy works as in the messages.

Examples:

```
Rijden Blue Drakes M16 1

za 31 okt 14:30 Uit tegen B.Z. '72 M16 1, Woerden
  Anna, Bram, Cas, nog 1 open
za 14 nov 12:00 Uit tegen Lions M16 1, Utrecht
  nog 3 open
```

```
Tafel en scheidsrechter Blue Drakes M16 1

za 31 okt 16:00 Blue Drakes X12 1 – De Dunckers X12 1
  Scheidsrechter: Gijs
  Tafel: Eva, nog 1 open
```

Stricter readings, confirmed by the coordinator (2026-10-31):

1. No team chosen for the current season: the buttons show, and the page says "No overview: no team chosen." The heading needs the team, so there is no text.
2. An empty overview has its heading, an empty line and the "Geen …" line, as the committee message.
3. The driving overview has no empty line between games (its example). The table and referee overview has one empty line between games (the form of the message of the week).
4. An unknown time in the driving overview: `za 14 nov, tijd nog niet bekend, Uit tegen Lions M16 1, Utrecht`, the form of the table and referee lines.

## Current state
- `crates/core/src/messages.rs`: `MessageGame`, `week_message`, `committee_message`, and the private `filled_and_open`, `game_block`, `places_of_team`, `duty_lines`, `week_games`, `short_date`, `clock`.
- `crates/server/src/api/messages.rs`: `MessageView`, `message_games` (the past games of this week and all coming games), `week_text`, `committee_text`, and `message_view`, which takes a function of type `fn(&SeasonGames, &PlacesOf, NaiveDate) -> Option<String>`.
- `schedule.coming`: today and later, in time order, unknown times last within a date.
- `counts` of a part: `past || (in_list && required)`. `required_duties` gives drivers only for an away game of the team with another home club, so "Uit tegen" is always true in the driving overview.
- `web/src/lib/components/MessagePage.svelte`: the page of a text (`title`, `back`, `empty`, `message`). `GET /api/duties` already has `season` and `current_season`.

## Steps

### Step 1: The two texts in the core
- **Goal**: the rules are complete and tested; the user sees no change.
- **Changes** in `crates/core/src/messages.rs`, exported from `lib.rs`:

  ```rust
  pub fn driving_overview(
      games: &[MessageGame<'_>],
      names: &BTreeMap<PlayerId, String>, // display_names
      team: TeamNumber,
      club_name: &str,
      team_name: &str,
      today: NaiveDate,
  ) -> String

  pub fn table_referee_overview(
      games: &[MessageGame<'_>],
      names: &BTreeMap<PlayerId, String>, // display_names
      club_name: &str,
      team_name: &str,
      today: NaiveDate,
  ) -> String
  ```

  Private helpers: `coming_games(games, today)` (date today or later, a stable sort by date), `drivers_of_team(game) -> Option<&DriverPart>` (the filter `part.counts && !part.places.is_empty()`, moved out of `game_block`), `opponent(game, team)` (moved out of `game_block`). `game_block` uses the last two. Each game of the driving overview is two lines; the games are joined with one newline. The parts of the text are joined with an empty line. `past` is false for all places, because only coming games are read.
- **Tests**, exact text where a text is given:
  - Driving: the example; an unknown time; no game gives the heading and "Geen ritten gepland."; left out: a game before today, 0 drivers, a coming game no longer in the game list, a game that no longer needs drivers; a game today stays, with "nog 1 open" and not "niemand"; a game today with a correction shows the planned player.
  - Table and referee: two games with an empty line between them; "Scheidsrechter:" before "Tafel:"; a kind without a place of the team has no line; places of other club teams are not in it; no game gives the heading and "Geen tafel- of scheidsrechtersdiensten gepland."; left out: a coming game no longer in the game list, a coming game no longer a home game of the club; dates sorted, an unknown time last within its date.

### Step 2: The two routes
- **Goal**: both texts through the API.
- **Changes**: in `crates/server/src/api/messages.rs` the functions `driving_text` and `table_referee_text` (the type of `committee_text`; `None` without a chosen team; they use `message_games` and `games.duties.names`) and the handlers `driving` (`GET /api/messages/driving`, operation `drivingOverview`) and `table_referee` (`GET /api/messages/table-referee`, operation `tableRefereeOverview`), both through `message_view`. Routes in `app.rs`, paths in `api/mod.rs`, then `just api-types`.
- **Tests** in `crates/server/tests/messages.rs`: both routes give 401 without a login and `{ "text": null }` without a team; stored data gives the driver of a game after the end of this week; stored data gives a table or referee place of the team and not a place of another club team; a team without duties gives each "Geen …" text.

### Step 3: The pages and the buttons
- **Goal**: the coordinator can copy both overviews.
- **Changes**: `loadDrivingOverview` and `loadTableRefereeOverview` in `web/src/lib/api/client.ts`; the pages `web/src/routes/duties/driving/` and `web/src/routes/duties/table-referee/` with `MessagePage.svelte` (`back` is `/duties`, `empty` is "No overview: no team chosen."); on `web/src/routes/duties/+page.svelte` the two buttons when `overview.season === overview.current_season`.
- **Tests**, with fixed server answers (decision 0025): `web/e2e/duties.e2e.ts`: the buttons show for the current season and not for an earlier season. `web/e2e/messages.e2e.ts`: each button opens its page and "Back" returns to "Duties"; "Copy" puts the text on the clipboard (one overview is enough: the component is shared); without a team the page says so; the page fits a phone and is accessible.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| The routes are `/api/messages/driving` and `/api/messages/table-referee` | `docs/language.md` calls a duty overview a message; all texts stay in one module with `message_view` | No |
| The rule "today and later" is in the core | One tested rule; the server keeps one list of games | No |
| `counts` is the only rule for what is in an overview | The same flag as the totals (decision 0022) | No |
| The overviews use `display_names` | The same first names as the message of the week | No |

These follow decisions 0024, 0025 and 0026. Close writes a record only if the build shows something new.

## Not in scope
- Counts or totals in the overviews; the report of what each player did.
- Overviews of an earlier season.
- A button or a flag on the start page.
- A choice of dates, or a limit on the length of the text.
- Changes to `GET /api/duties`.

## Test plan
- **Existing tests that must still pass**: the tests of `week_message` and `committee_message`, `crates/server/tests/messages.rs`, `web/e2e/messages.e2e.ts`, `web/e2e/duties.e2e.ts`.
- **New tests**: per step, see above.
- While working: `cargo test -p team_core`, `cargo test -p team --test messages`, `cd web && npx playwright test messages duties`. Once before a commit: `just check`.

## Compatibility and performance
Two new read-only routes; no migration. They read the same data as a message, at each request. The text can be long, up to all coming duties of the season.

## Progress
- [x] Step 1 – the two texts in the core
- [x] Step 2 – the two routes
- [x] Step 3 – the pages and the buttons
- [x] All checks pass

## Deviations
None from `functional.md`. The stricter readings are in the requirements summary.

Step 1 has three more small private helpers than planned, so that no code is repeated: `date_and_time` (the start of a line, also used by `duty_lines`), `drivers_and_open` and `overview`. The texts do not change.

## Review findings

Build review, 2026-10-31, on the uncommitted work against `main`. No bug against the intent of the spec was found, and the decisions hold: the texts are in the core (0001, 0024), the routes go through `message_view` and the pages through `MessagePage.svelte` (0026), and `counts` is the only rule for what is in an overview (0022, 0023). R1 and R2 are gaps in the tests; R3 to R5 are small.

Runs: `cargo test -p team_core messages` (38 passed), `cargo test -p team --test messages --test message_logs --test player_logs` (passed), `cargo clippy --workspace --all-targets -- -D warnings` (clean), `just api-types-check` (exit 0; this compares against the index, where `web/openapi.json` and `schema.d.ts` were staged), `npx playwright test messages duties` (132 passed, with the projects these depend on). `just check` as a whole was not run by this review.

- **R1 – The new routes are missing from the three lists of routes the older message routes are on.** Project rule: logs never contain player names. `crates/server/tests/message_logs.rs` reads `/api/messages/week` and `/api/messages/committee` only, and `crates/server/tests/player_logs.rs:102-103` the same, so no test checks the rule for the two overviews, which also hold names. The code logs no text today, so this is a missing guard, not a leak. `every_new_route_needs_a_login_and_changes_need_the_origin` in `crates/server/tests/players.rs:566-567` also lacks them, but `the_overviews_need_a_login_and_a_team` does cover the 401. Fix: add both routes to the two log tests, and to the list in `players.rs` to keep it whole; the header of `message_logs.rs` says "stories C and D". Checked by a search for `messages/committee` in `crates`, and by reading the tests.
  - 2026-10-31: Fixed: both routes are in the two log tests and in the list in `players.rs`.
- **R2 – Stricter reading 1 is half tested.** "No team chosen: the buttons show" has no test: `web/e2e/duties.e2e.ts` tests the buttons with a team, and the tests "without a team" in `web/e2e/messages.e2e.ts` open the page directly. The behaviour is right by a trace: `web/src/routes/duties/+page.svelte:68` looks only at the season, and `GET /api/duties` gives `season` and `current_season` without a team. The step did not list this test, but the reading was confirmed by the coordinator, and a later "hide without a team" would break it silently. Not run in a browser.
  - 2026-10-31: Fixed: a test in `web/e2e/choose.e2e.ts`, which runs before a team is chosen, also with the real answer of the server.
- **R3 – One test name promises more than it can check.** `the_overviews_are_in_time_order_with_unknown_times_last_in_a_date` in `crates/core/src/messages.rs` gives the games of one date already in that order, and `coming_games` only keeps it; the test fails on a lost date sort or an unstable sort, not on a change of the rule for unknown times. The rule itself is in `schedule` and tested there (`coming_games_are_in_time_order_with_unknown_times_last_in_their_date`, `crates/core/src/schedule.rs:281`), so nothing is unprotected; the comment in the test says this, the name does not. Fix: name it after what it checks. Checked by reading; no mutation was run.
  - 2026-10-31: Fixed: renamed to `the_overviews_sort_the_dates_and_keep_the_order_within_a_date`.
- **R4 – "Today and later" exists twice.** `schedule` already splits `coming` from `past` on the same comparison, and `coming_games` splits again the list that `message_games` built by adding the past games of this week, which are then dropped. It is harmless and was chosen in **Decisions** ("The rule is in the core"), and the two comparisons agree. A reader should know the sorting and filter in `coming_games` do nothing in production, apart from dropping those past games. No change needed; Close may keep it as is. Checked by a trace.
  - 2026-10-31: Not fixed: chosen in **Decisions**.
- **R5 – Documents to change at Close.** `docs/product/duties.md:50` and `docs/product/messages.md:9` say that the duty overviews "come in the next story"; `docs/architecture.md` (the `messages` module, "Messages", the `web/` routes) does not name the overviews, their routes or their pages. Expected at this stage, not a fault of the build. "All checks pass" in **Progress** is still open. Checked by a search for "overview" in `docs/product` and `docs/architecture.md`.
  - 2026-10-31: For Close.

Also checked, no finding: the claim in **Current state** that "Uit tegen" is always true (a coming game counts its drivers only when `required_duties` gives them, so the home club is another club); `past` is false in all overview lines; a game today shows the planned player, also after "Nobody" (`who_did` with `past` false); `town` is never empty (`filled_text("loc_plaats")` in `source/parse.rs`); the refactor of `duty_lines` gives the same text; the tests "left out" and "a game today" fail without their rule (by reading; no mutation was run).

### Accept review, 2026-10-31

**Verdict: accepted with notes.** Each acceptance criterion of story E is met. No concerning deviation. A1 needs a word from the coordinator; A2 is for Close.

Runs: the real server with the real pages on a temporary database and the fake game list source (`DEV_NOW` Wednesday 28 October 2026, team M16 1; database dropped afterwards), read through the API and in a browser; `cargo test -p team_core messages` (38 passed), `cargo test -p team --test messages --test message_logs --test player_logs` (passed), `cargo clippy --workspace --all-targets -- -D warnings` (clean), `npx playwright test messages duties` (133 passed). `just check` as a whole was not run, so the tick at "All checks pass" was not checked. The whole of this file was read in one go, so **Deviations** was seen before the run, not after; it holds only private helpers, and the verdict rests on the run.

| Acceptance criterion (story E) | Result | How checked |
|---|---|---|
| Two buttons for the current season, not for an earlier one; text, "Copy", "Back" to "Duties" | met | run in the browser: both links on `/duties`, none on `?season=2025-2026`, "Copied" after "Copy", "Back" gives `/duties` |
| Driving overview: coming away games with more than 0 places, in time order, in the form of the example | met | run: 3 games, then 2 after `drivers` 0 on the game of 21 November; "Anna, Bram" and "nog 2 open" |
| Table and referee overview in the form of the week part | met | run: the lines are the same as in the message of the week of the same day; "Scheidsrechter:" first; the place of M16 2 is not in it; "Eva B." and "Eva V." |
| No counts, no totals | met | run, and reading of the text functions |
| "Geen …" text, and "Copy" works | met | run for the text; `web/e2e/messages.e2e.ts` for the clipboard |
| The rule of the totals: a game no longer in the list or no longer needing duties is left out | met | core tests and a trace of `counts`; not run on the server (the fake source cannot change its list). The planned numbers on "Duties" agreed with the overview in the run |
| "tijd nog niet bekend" | met | run: `za 31 okt, tijd nog niet bekend, Blue Drakes X12 1 – De Dunckers X12 1` |
| A failed copy as in the messages | met | shared `MessagePage.svelte`, trace; the browser did not refuse in the run |

The stricter readings 1 to 4 are all faithful adaptations: each fills a point the spec left open, in the direction of its examples, and the coordinator confirmed them. Reading 1 was run: without a team both routes give `null`, and `web/e2e/choose.e2e.ts` passes with the real answer.

- **A1 – Neutral drift: a player who left the team, or is hidden, stays named on a coming duty.** The spec does not say. In the run, Anna left the team from 29 October and Bram was hidden, each with a drive on 7 November: the text stayed `Anna, Bram`, and no open place shows. It follows the rule of the totals (the place is still filled, and "Duties" still gives Anna 1 planned), so it is one rule everywhere, as the spec asks. But the overview goes to the team and the parents, who then read a driver who is gone, and the "nog N open" is too low. The message of the week reads the same data, so this is not of this story alone. Coordinator to decide: leave it (the coordinator swaps the player), or a line in `docs/ideas.md`. No correction needed for this story. Checked by a run.
  - 2026-10-31: Not fixed: the coordinator keeps the behaviour; a line in `docs/ideas.md`.
- **A2 – Neutral drift, for Close: documents beyond R5.** `docs/vision.md:44` and the change log say the duty overviews come next; R5 names only the product docs and `docs/architecture.md`. The row of story E in `../2026-10-18-duties/technical.md` says "design approved", and **Status** at the top of this file says "in progress". Checked by a search for "overview" in `docs/vision.md` and by reading.
  - 2026-10-31: For Close. The status in this file is now `done`.
- **A3 – Faithful adaptation: `Log in to see the message.` on the overview pages.** The text of `MessagePage.svelte` is shared; `docs/language.md` calls a duty overview a message, so the word is right. Checked by reading.
  - 2026-10-31: Not fixed: faithful (`docs/language.md`).
