# 0022 – Open items worked out from their source; only the check of a training is stored

**Status**: valid
**Date**: 2026-10-25
**Change**: [2026-10-25-start-page-and-season](../changes/done/2026-10-25-start-page-and-season/) (stories [Season](../changes/done/2026-10-25-season-page/), [checked trainings](../changes/done/2026-10-25-checked-trainings/), [the start page](../changes/done/2026-10-25-landing-page/))

## Context
The start page became a landing page: "Open items" (what still needs the coordinator), "Last" and
"Next"; a page "Season" shows all games and trainings of a season. The open items repeat facts
that other pages already show: the training days not opened ("Attendance"), the state of the game
list (the "Game list" card), the changes not seen (the list of changes). Two numbers for one fact
would make the coordinator distrust both. A past training that nobody looked at could not be told
apart from one where everyone was there, because only absences are stored (decision 0020). The
start page is opened often, on a phone, and must load in one request.

## Decision
- **Each open item comes from the function of its source, at each request.** Training days not opened: `team_core::not_opened`, the same function as the line on "Attendance". Trainings not checked: `team_core::not_checked`. The game list: `readings::status`, the same rule as the "Game list" card (`GET /api/readings/latest`). Changes not seen: the length of the same `api::changes::list` as `GET /api/changes` (read once, in `games_of`, which also decides the gone games). Nothing is counted on the side, as for the totals (decision 0020). `team_core::open_items` puts them in order.
- **Only the check of a training is stored.** `trainings.checked_at` (`NULL` = not checked; a second check keeps the first moment). "Change", "Cancel" and "Undo cancel" keep it; "Delete training" removes it with the row. A training of today or earlier that is not cancelled can be checked (`team_core::check_checking`); a checked training passes, also when it moved to a later date. At the release, migration 0010 checked every training before that day (Dutch date) once, so old trainings did not become open items.
- **The check never blocks a mark.** The check reads the training with `SELECT … FOR NO KEY UPDATE` and writes in one transaction: another check or "Undo check" waits, so the rule and the write cannot be split; a mark is not blocked, because an absence holds only `FOR KEY SHARE` on its training. "Undo check" is one `UPDATE`.
- **One route each.** `GET /api/season` and `GET /api/start`; the core puts games and trainings in one order (`season_events`), gives the ISO week, and works out "Last" (`last_training`) and "Next" (`next_events`). The front end only groups and formats.
- **The games of a season come from the team chosen for that season** (`games_of`); a club no longer chosen stays out of view. In the current season a gone game shows while its change is open; in an earlier season gone games are hidden.
- **Which seasons count.** The open items and "Last" cover the current season and, in July and August, also the season before (`open_item_seasons`), so the record of June stays in view after 1 July.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Open items from the functions of their sources (chosen) | One number everywhere; a fix of a rule fixes both places | The start page reads what several pages read (about 10 queries, 5–11 ms locally) |
| A stored list of open items, updated on each change | Cheap to read | Every action and reading must update it; it can drop out of date |
| `trainings.checked_at timestamptz` (chosen) | Keeps when it was checked at no cost | – |
| `checked boolean` | Simple | Loses the moment for no gain |
| Check past trainings once at the release (chosen) | No long list of open items on the first day | Trainings before the release are marked checked without a look; "Undo check" undoes one |
| A row lock `FOR UPDATE` for the check | Strict | Waits for every absence being saved (`FOR KEY SHARE`) |
| `FOR NO KEY UPDATE` (chosen) | The rule and the write stay together; marks go on | The difference between the two locks must be known (tests show both sides) |
| One route each for "Season" and the start page (chosen) | The order of events is a rule in the core; one request on the phone | New answer types |
| The front end merges several routes | No new route | Rules in TypeScript; several requests |

## Consequences
- A new open item (for example the duties: an away game without enough drivers, an open duty place, a possible duty game not handled) must come from the function that its own page uses, and is added to `team_core::open_items` and `OpenItemView`.
- A check is a fact the coordinator makes; nothing makes a training checked by itself after the release.
- `GET /api/schedule`, `GET /api/trainings/next` and `team_core::schedule_target` are gone; the view "Last season" on the start page is gone (the earlier season is on "Season").
- "Next" covers only the current season: in the last days of June its window can reach into the new season (an idea in `docs/ideas.md`).
- The front-end checks of "today" (for example `can_check`) come from the server: the browser's date is not the server's in the browser tests (`DEV_NOW`).
