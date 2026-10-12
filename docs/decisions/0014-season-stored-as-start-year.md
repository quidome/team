# 0014 – A season is stored as its start year

**Status**: valid
**Date**: 2026-10-12
**Change**: [2026-10-11-read-on-demand](../changes/done/2026-10-11-read-on-demand/)

## Context
Story 3a stores games and readings for each club and season. Story 2 stored the season of a
choice as text ("2026-2027") in `followed_teams.season`. Text can hold a wrong value, and sorts
as a number only by luck of its form. The game list gives dates and start times in Dutch local
time; a start time of 00:00 means "not known yet". Decision 0004 says a breaking database change
takes two releases, because old app code can run on new migrations during a rollout.

## Decision
- A season is stored as its start year (`season_year integer`): 2026 means "2026-2027". It is
  always shown as "2026-2027". The conversion is in one place: `Season::start_year` and
  `Season::from_start_year` in `crates/core`.
- New tables (`games`, `readings`) have only `season_year`.
- `followed_teams` gets `season_year` as a computed column from the text `season`, with a unique
  index. New code reads only `season_year`; saving still writes the text `season`, so the released
  code of story 2 keeps working. The text column goes in a later release.
- Dates and start times of games are stored as Dutch local `date` and `time`, not as
  `timestamptz`. A start time that is not known is `NULL`.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Start year as `integer` | Sorts and compares as a number; cannot hold a wrong text | A conversion at the edges |
| Text "2026-2027" (as story 2) | Readable in the database | Can hold any text; ordering depends on the form |
| A date range | Exact | More than the product needs; a season always starts on 1 July |
| Games as `timestamptz` | One moment for each game | A shown time can shift with time zone rules; "not known" needs a separate flag |

## Consequences
- Migration `0003_games` adds `followed_teams.season_year`; a later migration drops
  `followed_teams.season` once no released code writes it (technical debt in
  `docs/architecture.md`).
- A shown date or time of a game is exactly what the game list gave.
