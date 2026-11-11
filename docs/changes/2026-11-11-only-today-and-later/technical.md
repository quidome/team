# Technical spec: Only today and later on the start page

**Status**: approved
**Size**: story
**Change**: docs/changes/2026-11-11-only-today-and-later/
**Date**: 2026-11-11

## Objective
Remove "the last duty games" from the start page, so its list of events holds only today and
later and a past event shows only through the open items.

## Requirements summary
From the epic's `functional.md`, story 4 (`docs/changes/2026-11-11-checked-for-every-event/`):
- The part "last duty games" is not shown: a past event is shown only through the open items
  (story 3).
- The list has the events from today until the Sunday of next week, as now, with the same rows
  and marks. An event of today that is done (for example a checked training) is lighter, as now.
- Without coming events: "No coming events.", as now.

## Current state
- Core `output/start_page.rs`: `last_games` (the duty games of the newest past date) with three
  tests; `event_look(event, places, game_checked, today)` makes an event before `today` lighter (only
  the last duty games are before today), with a test.
- Server `api/start.rs`: `StartAnswer.last_games` and `last_games_season`, built from `last_games`,
  in July and August from the season before (an extra read of its games and places).
- Front end `routes/+page.svelte`: the part `last-games` above the list.
- Tests: `tests/games.rs` (`the_last_duty_games_are_…`), `tests/not_played.rs`
  (`a_not_played_duty_game_stays_in_the_last_duty_games_and_on_season`), `tests/checked.rs`
  (a line on `last_games`), `tests/start.rs` (July: `last_games` empty); `web/e2e/duties.e2e.ts`
  (`the last duty games come above the coming events…`), `web/e2e/start.e2e.ts` (a past training
  is not in the list).

## Steps

### Step 1: Core and server
- **Goal**: no last duty games in the rules and the answer.
- **Changes**: remove `last_games` and its tests; `event_look(event, places, game_checked)`
  without `today` and without "an event before today is lighter" (its test goes); remove
  `StartAnswer.last_games`, `last_games_season` and the reads that only served them (the read of
  the season before in July and August stays for the open items); `just api-types`.
- **Tests**: remove the server test of the last duty games (`games.rs`) and the line in
  `checked.rs`; `not_played.rs`: a not played duty game is not in "duty games not checked" and
  "Season" shows it as not played; `start.rs` (July) without `last_games`; new: the list of the
  start page has no past event, also when a past duty game is not checked (it is only in "duty
  games not checked").

### Step 2: Front end and checks
- **Goal**: the page without the part; all checks pass.
- **Changes**: remove the part `last-games` from `routes/+page.svelte` and its comment; a
  screenshot of the start page; `just check`.
- **Tests**: remove the browser test of the last duty games (`duties.e2e.ts`); the test of a past
  training in `start.e2e.ts` also covers a past duty game with a place of the team.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| `event_look` loses `today` and "before today" | The list holds only today and later, so the condition can never be true | no |

## Not in scope
- Any change to the open items or to the rows of coming events.

## Test plan
- **Existing tests that must still pass**: all, with the changes above.
- **New tests**: per step.

## Compatibility and performance
- `GET /api/start` loses two fields; only the start page reads it. In July and August the start
  page reads the season before once instead of twice.

## Progress
- [ ] Step 1: core and server
- [ ] Step 2: front end and checks
- [ ] All checks pass

## Deviations

## Review findings
