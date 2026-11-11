# Technical spec: A quieter start page

**Status**: in progress
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
    `!not_entered(places)`; a game that left the game list: normal.
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
    false for a game not played or gone; equal to "the committee text has lines" on the cases of
    the existing committee tests.
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
- [ ] All checks pass

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
