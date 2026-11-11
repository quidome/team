# Technical spec: A quieter start page

**Status**: done
**Size**: story
**Change**: docs/changes/2026-11-10-quieter-start-page/
**Date**: 2026-11-10

## Objective
Make the start page show only what needs the coordinator and what comes next, quietly: open items
as rows, the coming events as one list where done events are lighter and the games of the team
stand out, the duty games of the last date with duties, the messages only when they have
something, and a short header with fewer links.

## Requirements summary
From `functional.md` (approved 2026-11-10, two spec reviews):

1. **Open items as quiet rows.** Same kinds, texts, order and tap targets as now. One row each:
   orange dot, text, "›"; no card, no heading. An item of more games keeps its button that opens
   its games below it. None: one grey row "Nothing needs you.".
2. **Coming events as one quiet list.** Same events and range as "Next" now (today until the
   Sunday of next week; else the first coming event not cancelled; else "No coming events.").
   No heading "Next", no cards; grouped by date (small grey heading), rows in time order with the
   start time on the left ("time not known yet").
   - Game of the team: soft orange background and orange bar; first line "Home game against
     <opponent>" or "Away game at <opponent>" (home: in the club's own hall, also as the away
     side against another club team); next line hall and town, then the count / "Not played" /
     "Checked"; then the places of the team with today's text.
   - Other game with places of the team: thin blue bar, no background; both teams; the places
     with today's text; no hall and town (always the club's own hall).
   - Training: no bar; "Training" and its status as now.
   - **Lighter** (grey text, lighter bar) when nothing is left to do: a training checked or
     cancelled; a game of the team checked or not played; another game not played, or with all
     places of the team filled and its schedule entered. A home game of the club with the
     schedule not entered shows "Schedule not entered" and is normal. All else normal; a gone game
     stays crossed out. Warnings stay on the row. Taps and "Back" as now. Rows at least 44 px;
     readable in light and dark mode.
3. **Past events only when they need the coordinator.** No "Last", no past training, no past game
   of the team in the list (they are open items until checked). At the top of the list: the
   games of the newest date before today with places of the team that are **not** games of the
   team, under their date heading, blue bar, places, lighter; seasons as "Last" now (current; in
   July and August, when it has none, the season before). A newer such date replaces them. A tap
   opens the game; "Back" returns.
4. **Messages.** Below the events: "Message of the week ›" by today's rule; "Committee message ›"
   only when this week has table or referee places of the team (the rule of the lines
   "Tafel en scheidsrechter:"). No heading. "Duties" has the button "Committee message" next to
   the two overviews, current season only, whenever a team is chosen. "Back" on a message returns
   to the page it was opened from.
5. **Header and links.** Header: team name and today's date ("Tuesday 10 November"); in July and
   August also the season ("Blue Drakes M16 1 · 2026-2027"); no "Change". In a new season without
   a team: only the date, the choose step below. Links at the bottom: Season · Attendance ·
   Duties · Report · More. New page "More": Team, Players, Settings, and "Back". Settings has
   "Change club or team" (to `/choose`).

## Current state
- **Core** `crates/core/src/output/start_page.rs`: `open_items`, `schedule_not_entered`,
  `open_places_games`, `open_item_seasons`, `last_training` (newest past training row),
  `last_games` (games of the newest past date with places of the team, team games included),
  `next_events`, `next_until`; `PlacesOf = BTreeMap<i64, GamePlaces>`.
  `crates/core/src/contribution/duties.rs`: `GamePlaces`, `not_entered`, `open_places`,
  `open_by_kind`, `has_team_places`.
  `crates/core/src/output/messages.rs`: `week_message`, `committee_message` (always a text,
  decision 0026), private `game_block` (the "Thuis"/"Uit" rule: `game.home.club == club`),
  `opponent`, `duty_lines`, `places_of_team`, `week_games`.
- **Server** `crates/server/src/api/start.rs`: `StartAnswer { open_items, last, next,
  last_games, last_games_season, messages: MessagesView { week, committee } }`; `next` via
  `coming_view`, `last_games` via `event_view_with`, both `EventView` from `api/season.rs`
  (shared with "Season"). `api/messages.rs`: `week_text`, `committee_text` (`Some` with a team).
- **Front end**: `web/src/routes/+page.svelte` (header with "Change" to `/choose`, the parts,
  `EventRow.svelte` shared with `SeasonEvents.svelte`), `src/lib/start.ts` (`openItemText`,
  `openItemLink`, `gameLink`, `trainingLink`, `gameItemText`), `src/lib/season.ts`
  (`groupByDate`), `src/lib/schedule.ts` (`formatDay`), `src/lib/duties.ts` (`rowPlacesText`,
  `placesWarnings`), `src/lib/attendance.ts` (`countText`), `src/lib/trainings.ts`.
  `routes/messages/committee/+page.svelte` passes `back={resolve('/')}` to `MessagePage.svelte`.
  `routes/duties/+page.svelte` has the overview buttons (`data-testid="overviews"`) for the
  current season. Colours in `src/routes/layout.css` (`--team`, `--team-soft`, light and dark).

## Steps

### Step 1: The core rules
- **Goal**: the rules the start page needs, tested, with nothing changed for the user.
- **Changes**:
  - New module in `output` (for example `output/side.rs`): `pub enum Side { Home, Away }` and
    `pub fn team_side<'g>(game: &'g Game, club: ClubNumber, team: TeamNumber) -> (Side, &'g str)`
    (home when `game.home.club == club`; the opponent is the other team, as `opponent` now).
    `game_block` in `messages.rs` uses it; `opponent` moves into it.
  - `start_page.rs`: `pub enum EventLook { Normal, Lighter }` and
    `pub fn event_look(event: &Event<'_>, places: Option<&GamePlaces>, game_checked: bool) -> EventLook`:
    training lighter when checked or cancelled; game of the team lighter when `game_checked` or
    not played; other game lighter when not played, or when `open_places(places) == 0` and
    `!not_entered(places)`; another game that left the game list: normal. (Changed at the reviews:
    an event before today is always lighter; a gone game that is not played, or a checked gone
    game of the team, is lighter.)
  - `messages.rs`: `pub fn week_has_duty_places(games: &[MessageGame<'_>], today: NaiveDate) -> bool`:
    true when a game of `week_games` gives `duty_lines` (the same games as the text).
  - Export the new names flat from `team_core` (the server uses only flat names).
- **Tests** (unit tests next to the code):
  - `event_look`: lighter for a checked training, a cancelled training, a checked game of the
    team, a game of the team not played, another game not played, another game filled and
    entered; normal for a coming training, a training day not opened, a coming game of the team
    with all places filled, another game with an open place, another game filled but not entered,
    a game no longer in the game list.
  - `team_side`: home in the club's hall; away; the team as away side against another club team
    in the club's hall gives `Home` and that team as opponent.
  - `week_has_duty_places`: true with places this week, also on a passed day; false without;
    false for a game not played or gone; equal to "the committee text has lines" on each of its
    own cases (the existing committee tests do not call the rule).
  - The existing tests of the message of the week pass unchanged.
  - `cargo test -p team_core`; `crates/core/tests/domains.rs` passes (no new use between domains).

### Step 2: The new places
- **Goal**: "More", "Change club or team" and the committee message on "Duties" exist before the
  start page drops its links.
- **Changes**:
  - New route `web/src/routes/more/+page.svelte`: heading "More", links Team, Players, Settings
    (`resolve()`), "Back" to `/`.
  - `routes/settings/+page.svelte`: a link "Change club or team" to `/choose`.
  - `routes/duties/+page.svelte`: in the overview buttons, "Committee message" to
    `/messages/committee?from=duties`.
  - `routes/messages/committee/+page.svelte` (and its `+page.ts` if the query is read there):
    `back` is `/duties` when `from=duties`, else `/`.
- **Tests** (browser): "More" shows the three links and "Back" returns to the start page;
  Settings has "Change club or team" that opens the choice; "Duties" has "Committee message",
  its page has the text, "Back" returns to "Duties". `cd web && npx playwright test <file>` for
  each changed file.

### Step 3: The start page
- **Goal**: the new start page, on the new answer.
- **Changes**:
  - Core: `last_games` leaves out `ScheduleReason::TeamGame` before it takes the newest date.
    Remove `last_training` and its tests.
  - Server `api/start.rs`:
    ```rust
    pub struct StartEvent {
        pub event: EventView,          // unchanged, shared with "Season"
        pub look: EventLookView,       // "normal" | "lighter"
        pub side: Option<SideView>,    // "home" | "away"; only a game of the team
        pub opponent: Option<String>,  // only a game of the team
    }
    ```
    `StartAnswer`: remove `last`; `next` and `last_games` become `Vec<StartEvent>` (`look` from
    `event_look` with the game's places and its checked state, the same data `event_view_with`
    reads); add `today: String` (Dutch date) and `show_season: bool`
    (`open_item_seasons(today).len() > 1`). `messages.committee` from a new
    `committee_offered(games, places, today) -> bool` in `api/messages.rs` (false without a team,
    else `week_has_duty_places`). `GET /api/messages/committee` is unchanged.
  - `just api-types` (OpenAPI and `web/src/lib/api/schema.d.ts`).
  - Front end: new `src/lib/components/StartEventRow.svelte` for a `StartEvent` (cases of story
    2, texts from the existing helpers, links with `?from=start` as now, at least 44 px);
    `routes/+page.svelte`: header (team, long date, season when `show_season`; choose step as
    now), open items as rows (dot, text, "›", the expand button of an item of more games kept),
    "Nothing needs you.", the last duty games then the coming events grouped by date
    (`groupByDate`), "Message of the week ›" / "Committee message ›", links Season · Attendance ·
    Duties · Report · More. A long date format next to `formatDay` in `src/lib/schedule.ts`.
    A blue colour (for example `--duty`) for light and dark in `layout.css`.
  - The comment at the top of `+page.svelte` and of `start.rs` follows the new parts.
- **Tests**:
  - Server `tests/start.rs`: remove `last_is_the_newest_past_training_also_when_checked` and
    `last_shows_the_absent_players_of_its_training`; change
    `in_july_the_open_items_and_last_cover_the_season_before` (open items and last duty games of
    the season before); new: `today` and `show_season` (October false, July true); a game of the
    team in `next` has `side`, `opponent` and `look`; the committee flag false without places this
    week and true with places. `tests/games.rs:830`: the games of the team are left out.
    `tests/not_played.rs:489`: a duty game not played stays in `last_games`, lighter.
    `tests/messages.rs`: unchanged and passing.
  - Browser: `start.e2e.ts` (the first test: header, rows, "Nothing needs you." where it
    applies, links; the test of "Last" replaced by the last duty games: at the top, lighter, a tap
    opens the game, "Back" returns), `duties.e2e.ts:375`, `not-played.e2e.ts:58`,
    `messages.e2e.ts:83, :100` (committee link only with places; on "Duties"), `choose.e2e.ts:55,
    :260` ("Change club or team" on Settings), `players.e2e.ts:21–23` (through "More");
    `report.e2e.ts` and `season.e2e.ts` only if their link lists are the start page's. A past
    duty game is made with the existing helpers when the data at the test date has none.
  - Run the full browser chain once at the end of the step (the files share one database).

### Step 4: The check of the look
- **Goal**: the page is readable and calm in light and dark mode.
- **Changes**: compute the contrast of the new colours (the blue bar at least 3:1 against the
  page; lighter text at least 4.5:1); screenshots at 390 × 844 in light and dark mode with a
  quiet week and a busy Saturday; fix what is not readable.
- **Tests**: none new; `just check` before the commit.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| "Lighter" is decided in the core (`event_look`) | Product rules live in the core (decision 0001); the front end has only browser tests | no |
| A `StartEvent` around the unchanged `EventView` | `EventView` is shared with "Season", which must not change or carry unused fields | no |
| A new `StartEventRow.svelte`; `EventRow.svelte` stays | One shared row would change "Season" by accident; the new row can serve "Season" later | no |
| `last_training` and `last` are removed | Unused data must not stay behind (warning from Discover) | no |
| One `team_side` for the message of the week and the start page | One rule for "home game"; the message tests guard it | no |
| The committee message on the start page only in a week with places; always on "Duties" | A quieter start page; "Duties" keeps the reason of 0026 (the coordinator can always tell the committee "nothing this week") | yes: 0034, replaces the part of 0026 about the button in every week (written at Close) |
| "Back" of the committee message from `?from=duties` | The same pattern as `?from=start` on the pages of a game and a training | no |

## Not in scope
- "Season" and `EventRow.svelte` (an idea in `docs/ideas.md`).
- A bottom bar; a unit test tool for the front end.
- Gone games lighter.
- The text of the messages; the rules of the open items.

## Test plan
- **Existing tests that must still pass**: all core tests (the message tests unchanged),
  `crates/core/tests/domains.rs`, the server tests not named above, the browser tests not named
  above.
- **New tests**: per step, above.

## Compatibility and performance
- `GET /api/start` changes shape (`last` gone; `next` and `last_games` become `StartEvent`;
  `today`, `show_season` new). Only the start page reads it; the front end and the server ship
  in one image, so there is no older client.
- No migration, no new table, no new write. The start page does the same reads as now; the new
  rules are in memory.

## Progress
- [x] Step 1: the core rules (2026-11-10)
- [x] Step 2: the new places (2026-11-10)
- [x] Step 3: the start page (2026-11-11)
- [x] Step 4: the check of the look (2026-11-11). Contrast (WCAG, from the colours in
  `layout.css`), light / dark: lighter text on the page 5.76 / 7.70 and on the orange row 5.25 /
  6.14 (need 4.5); blue bar 5.74 / 7.75, orange bar on the orange row 4.38 / 6.28, open item dot
  4.81 / 7.88 (need 3); the message links 4.81 / 7.28 (need 4.5). The lighter bars are decoration
  only: the grey text carries "done". Screenshots at 390 × 844 of a quiet week (the test data)
  and a busy Saturday (real games of the test data with changed dates and places) in light and
  dark mode: all readable, nothing changed.
- [x] All checks pass

## Deviations
None. Design reads the spec as written; points the spec leaves to Design: a game that left the
game list is not lighter (the spec lists all lighter cases and says "All other events have
normal text"); the header date comes from the server (`today`), so it follows the test clock.

- 2026-11-10, step 1 (Implement): the enum is `HomeOrAway`, not `Side`, in the module
  `output/home_or_away.rs`: `team_core` already exports `Side` (one side of a game,
  `events::game::Side`), and the server uses only flat names. `opponent` moved into that module
  too, because the driving overview also uses it. Only names; no behaviour changes.
- 2026-11-10, step 2 (Implement): "Committee message" on "Duties" shows with the two duty
  overviews, also without a chosen team; its page then says "No message: no team chosen.". The
  spec says "whenever a team is chosen" and "as it shows no overviews", but the overviews do show
  without a team (`docs/product/messages.md`), and "Duties" does not know whether a team is
  chosen. Following the overviews keeps the three buttons together; the other way needs a new
  field in the answer of "Duties". Accepted by the coordinator on 2026-11-10.
- 2026-11-11, step 3 (Implement): `SeasonTrainings::state` in `api/season.rs` is removed too:
  only the last training used it. The dot and the arrow of an open item come from CSS, so the
  text of the row (and of the browser tests) stays only its words. The browser tests of a game
  marked "Not played" and of the last duty games replace the answer of the start page, because
  the test clock (12 October 2026) has no coming game of the team in the two weeks; the server
  tests check the same with stored data.

## Review findings

### Accept review (2026-11-11, `/review accept`, fresh agent)
No extra text in the arguments. Read: `change.md`, `functional.md` (with both spec reviews),
`docs/language.md`, `product-workflow`, the diff `main...quieter-start-page` (core, server, front
end, tests), then this file (the Deviations last). Run: `cargo test -p team_core` (363 passed) and
`cargo test -p team --test start --test not_played --test games --test messages` (all passed).
Not run: the app itself and the browser tests (they share one database with other work in this
session); the look in light and dark mode is taken from step 4 and "not checked" by this review.

| # | Criterion / open point | What was built | Sort |
|---|---|---|---|
| A1 | Story 3: the last duty games "are lighter (the date has passed)"; this file, requirement 3, says the same | They go through `event_look`, the rule of coming events. A past duty game with a place of the team that has no player, or whose schedule was never entered, is **normal** (bold, full bar, "Schedule not entered" on the row). `crates/server/tests/not_played.rs:513` pins this: `assert_eq!(look(…, "last_games", id), "normal", "an open place")`. Not under Deviations. For a past game the reason the spec gives for "normal" does not hold: the open items "open places" and "schedule not entered" cover only coming games, so on Monday a normal-weight row from Saturday stands above the coming events with no open item behind it. Checked by a trace (`start.rs` `last_games` → `start_event` → `event_look`; `open_places_games`/`schedule_not_entered` take coming games only) and by reading that test. | **concerning deviation** |
| A2 | Story 2: a game of the team, "the line below has the hall and town, then … the count" (example "De Trits, Baarn · 10 no answer") | Hall and town with "· Not played" / "· Checked" on one line; the count on its own line below. Checked by reading `StartEventRow.svelte`. One line more per game of the team; the content is the same. | neutral drift |
| A3 | Story 1: each item "an orange dot, the text, and ›" | An item of more games shows "▾" / "▴" instead of "›", because a tap opens its list instead of a page. Checked by reading `+page.svelte`. | faithful adaptation |
| A4 | Story 5: "More" → "Back" returns to the start page | As specified. Team, Players and Settings opened from "More", and the choice of club and team opened from "Settings", still go "Back" to the start page, not to "More" or "Settings". The spec says nothing about them. Checked by a search for `Back` in those routes. | neutral drift |
| A5 | Story 4: "Duties" shows the committee message "whenever a team is chosen" | Shown with the overviews, also without a team (page says "No message: no team chosen."). The spec's premise ("as it shows no overviews") was not true; accepted by the coordinator (Deviations, step 2). Earlier season: no button, as specified. Checked by reading `duties/+page.svelte`. | faithful adaptation |
| A6 | Story 2: a game no longer in the game list | Normal, crossed out, "no longer in the game list" below; follows "All other events have normal text". Checked in `event_look` and its test. | faithful adaptation |
| A7 | Story 4: "Committee message ›" only in a week with table or referee places of the team | `week_has_duty_places` uses the same games as the committee text, and its test asserts both agree on each case (also a passed day, a gone game, "not played", next week). Checked by reading `messages.rs` and the run. | as meant |
| A8 | Story 5: header, date, season in July and August, new season without a team, links | Team name and "Tuesday 10 November" (checked with `Intl` in node), season when `show_season` (`open_item_seasons(today).len() > 1`), in `team_needed` only the date above the choose step; links Season · Attendance · Duties · Report · More. Checked by reading `+page.svelte` and `start.rs`. | as meant |
| A9 | Stories 1–3, other criteria (kinds and order of open items, "Nothing needs you.", range of events, grouping by date, bars and background, lighter rules of coming events, games of the team left out of the last duty games, taps with `?from=start`, rows at least 44 px) | Match. Checked by reading the code and the core tests of `event_look` and `last_games`; the 44 px by the class `min-h-11`. | as meant |

Smaller, for Close: `EventRow.svelte` still says it is the row "of Season and the start page" and
keeps the prop `gameFrom` that only the start page passed (search `gameFrom` in `web/src`: only
its own declaration and use).
- 2026-11-11: fixed with R4 (comment changed, `gameFrom` removed).

**Answers (2026-11-11, Implement).**
- A1: fixed, as the coordinator chose ("always lighter"): `event_look` takes `today`, and an event
  before today is lighter. New core test
  `a_past_duty_game_is_lighter_also_with_an_open_place_or_no_schedule`; `not_played.rs` now
  expects "lighter" with an open place.
- A2, A3, A4, A6: not changed, accepted as they are (the review says they can stay).
- A5: accepted earlier by the coordinator.

**Verdict: needs correction** (next: `/discover review:`), for A1 only. Either the build follows
the spec (the last duty games are always lighter, and the test at `not_played.rs:513` changes), or
the coordinator chooses that a past duty game with an empty place or a schedule not entered keeps
normal weight, and the spec says so. The rest is accepted; A2 and A4 can stay as they are.

### Build review (2026-11-11, `/review build`, fresh agent)
No extra text in the arguments. Read: `product-workflow`, `docs/language.md`, the change folder,
`docs/architecture.md`, decisions 0001, 0022, 0026 and 0027, `CLAUDE.md`, and the diff
`main...quieter-start-page` (no changes outside it at the start). The accept review above was
written while this review ran; R1 was found before reading it. Run: `cargo test -p team_core`
(363 + 7 domains, passed), `cargo test -p team --test start --test not_played --test games
--test messages --test checked --test game_attendance` (all passed), `cargo clippy --workspace
--all-targets -- -D warnings` (clean), `svelte-check` (0 errors, 0 warnings). Not run: the
browser tests, `api-types-check`, `check-sqlx`.

What the checks cannot see, checked and in order: "lighter" is decided in the core
(`event_look`), "home or away" is one rule (`team_side`) for the message of the week and the start
page, and the committee flag reuses `week_games` and `places_of_team`, so nothing is rebuilt.
`output` now uses `contribution::duties::open_places` and `events`: allowed by the domain map
(and `domains.rs` passes). `EventView` and `EventRow.svelte` are unchanged for "Season". A `Kept`
game in `last_games` is not a game of the team (`in_schedule` gave `None`), so leaving out only
`TeamGame` is complete. Checked by reading `schedule.rs` and the diff.

| # | Finding | How checked | Weight |
|---|---|---|---|
| R1 | **The last duty games are not always lighter.** Story 3 and requirement 3 say they are lighter (the date has passed). `start.rs` passes them through `start_event` → `event_look`, the rule for coming events, so a past duty game with a place of the team without a player, or whose schedule was not entered, is `normal`. `tests/not_played.rs:513` asserts `"normal", "an open place"`, so the test pins the difference; it is not under Deviations. The browser test of the last duty games (`duties.e2e.ts`) sets `look: 'lighter'` itself in `page.route`, so it cannot see it. Same as A1. Fix in the core (for example a `past` input to `event_look`, or a rule for the last duty games next to `last_games`) with a unit test, or the spec changes. | Trace and run of `a_not_played_duty_game_stays_in_the_last_duty_games_and_on_season` | must fix or decide |
| R2 | **The doc comment of `event_look` is false for gone games.** It says "All else is `Normal`, also a game no longer in the game list", and Step 1 says "a game that left the game list: normal". The arms `not_played => true` and `TeamGame => game_checked` come before the `in_list` test, so a gone game marked "Not played", and a checked game of the team that later left the game list, are `Lighter` (and crossed out). Only a gone duty game is tested. Either the comment says so, or the `in_list` test comes first; then a test for each. | Reading the `match` in `start_page.rs` and its tests | small |
| R3 | **A claim of Step 1 is wider than the test.** "`week_has_duty_places` … equal to 'the committee text has lines' on the cases of the existing committee tests": the agreement assert (`has_duty_places`) runs only in the two new tests (9 cases); the existing committee tests do not call it. The cases it has are good (passed day, gone, not played, next week, other teams, drivers). Also no API test shows `look: "lighter"` for a checked training, a checked game of the team or a filled and entered duty game: the server tests cover only "not played" (lighter) and "nothing checked" (normal); the mapping in `start_event` (`checked` from `EventView.game.checked`) is covered for those through the core tests only. | Search `has_duty_places(` in `messages.rs`; search `"look"` in `crates/server/tests` | small |
| R4 | **Comments the diff made false.** `crates/core/src/output/start_page.rs:982` ("not in \"Last\""), `crates/server/tests/start.rs:2` ("\"Last\" and \"Next\""), `crates/server/tests/checked.rs:385-386` ("the rows of \"Season\" and \"Last\" say so (a driver place puts the game in \"Last\")": the test now asserts the game is not there). `EventRow.svelte` still says it is the row of the start page and keeps `gameFrom` (as in the accept review). Code comments and the schema cite decision 0034, which is written only at Close. | Search `"Last"\|0034\|gameFrom` in `crates` and `web/src` | small |
| R5 | **For Close: `docs/architecture.md`** names the old start page in lines 18 (`"Open items", "Last" and "Next"`, `EventRow.svelte` "on Season and the start page", the buttons in the part "Messages", the links to `/team`, `/players`, `/settings`), 64 (`last_training`, `last_games` "all games"), 106 (`last_training` in `GET /api/start`), 107, 128 and 136 ("Last"), and 244 ("Last" in the reads done twice). Not in it yet: `home_or_away`, `event_look`, `week_has_duty_places`, `StartEvent`, `StartEventRow.svelte`, `/more`. | Search `Last\|last_training\|EventRow` in `docs/architecture.md` | for Close |

**Verdict:** the structure follows the project's rules; R1 is a bug against the spec (or a choice
the spec must state) and must be settled before Close; R2–R4 are small fixes; R5 is work for Close.

**Answers (2026-11-11, Implement).**
- R1: fixed with A1 (see above): the rule is in the core (`event_look` with `today`), with a unit
  test and the changed server test.
- R2: fixed. The comment of `event_look` now names every case (an event before today; a game not
  played, also when gone; a checked game of the team, also when gone; another game that left the
  game list: normal). New core test
  `a_game_that_left_the_game_list_is_lighter_when_not_played_or_checked`. Step 1 above corrected.
- R3: fixed. The claim of Step 1 is corrected. New server test
  `a_row_of_the_start_page_is_lighter_when_nothing_is_left_to_do` (`tests/games.rs`): a duty game
  with an open place, then filled but not entered (normal), then entered (lighter); a training of
  today, then checked (lighter). A checked game of the team is covered by the core test only: the
  test data has no game of the team that can be checked in the window of the list.
- R4: fixed: the comments in `start_page.rs`, `tests/start.rs` and `tests/checked.rs` no longer
  name "Last"; `EventRow.svelte` says it is the row of "Season" and no longer has `gameFrom`. The
  mentions of decision 0034 stay: Close writes it.
- R5: not fixed now: work for Close (`docs/architecture.md`).
