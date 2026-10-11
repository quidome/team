# 0012 – The choice of club and team is kept for each season

**Status**: valid
**Date**: 2026-10-11
**Change**: [2026-10-11-choose-club-and-team](../changes/done/2026-10-11-choose-club-and-team/)

## Context
The coordinator chooses the club and the team that the tool reads games for. A youth team moves
up an age group each season, so the team is chosen again each season, while the club stays. The
coordinator wants the games and choices of earlier seasons kept (for example 2025-2026 next to
2026-2027), and may later manage more than one team. Until a team is chosen in a new season, the
tool must still know last season's team (story 3 shows its games until then). The source's team
numbers stay the same between seasons, so a team number alone does not say which season it is.

## Decision
- Table `followed_teams`: one row for each season with the club (number, name, town), the team (number and short name, both empty while the season has no games), the season and the time of the choice (`chosen_at`). `UNIQUE (season)`.
- A save replaces the row of its own season only; rows of other seasons are never changed or removed.
- The state of the choice is read from two rows: the row of the current season, and the newest row before it. A row of a later season (a clock that went back) is never read.
- No row for the current season, with an earlier row: the club stays and a team is needed ("new season"). A row with no team: a team is needed ("no games yet"). In both cases the earlier choice is given as `last`, also after the club was saved for an empty new season.
- `chosen_at` is for the later stories: the first reading after a choice records no changes, and unseen changes of the old choice are removed.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| One row for each season | Seasons kept; last season's team always known; more teams later only removes `UNIQUE (season)` and adds a link to the coordinator | Two reads for the state |
| One single row, overwritten | Simplest | Loses last season at each change; blocks more teams |
| One row for each season with a "current" mark | Explicit | With one team for each season the mark holds no extra information |

## Consequences
- Story 3 can show last season's games from `last` until the team is chosen.
- More teams for one coordinator, or more coordinators, is a data change that keeps existing rows (idea in `docs/ideas.md`, for `/vision`).
- Choices are never deleted by the tool.
