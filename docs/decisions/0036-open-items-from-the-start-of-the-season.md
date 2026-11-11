# 0036 – Open items from the start of the season

**Status**: valid
**Date**: 2026-11-11
**Change**: [2026-11-11-open-items-from-the-start-of-the-season](../changes/done/2026-11-11-open-items-from-the-start-of-the-season/) (story 3 of [Checked for every event](../changes/2026-11-11-checked-for-every-event/))

## Context
Decision 0032 stored the day of the release of "Checked" for games (`game_check_start`, 4
November 2026): from that day a past game of the team that was not checked was an open item;
older games never were, and showed their list of who comes closed. The reason was that answers at
games could only be entered from that release. On 2026-11-11 the coordinator had entered the older
games by hand and still had to search for the ones that were not checked; the report counts a
game only when it is checked. "Checked" now holds for every event, also a duty game (decision
0035).

## Decision
- **No start date:** every past game of the team of the season that is not checked, not "not
  played" and still in the game list is an open item ("games not checked"), from the start of the
  season. `game_check_start` is dropped (migration `0018_drop_game_check_start.sql`); the list of a
  past game is closed only when it is checked or not played. This replaces the part of decision
  0032 about the release day.
- **A new open item "duty games not checked"** right after it: every past duty game of the season
  that is not checked (`team_core::duty_games_not_checked`, with `check_kind` and `game_checked` of
  decision 0035), still in the game list and not "not played".
- **Both cover the seasons of the open items** (in July and August also the season before), are
  in time order (date, start time, id), and carry their games: one game opens its page, more open
  their list below the item (`GameItemView`, looked up by season and id).
- **The report's line "N games not checked"** follows the same rule; duty games are not in it (they
  do not change the numbers of the report).

## Options considered
| Option | For | Against |
|--------|-----|---------|
| From the start of the season (chosen) | Every gap is found; nothing to keep up to date; the same rule as trainings | Many items at once after the release, or after a team is chosen in mid-season |
| Keep the release day | Old games without answers never ask | The coordinator must search for the gaps by hand |
| A start date per season in the settings | The coordinator chooses | One more setting to keep up to date |

## Consequences
- After the release, and after a team is chosen in the middle of a season, every past game that is
  not checked asks at once; the item stays one row with its list.
- "Mark as checked" on an old game without answers makes every player absent; the administration
  panel warns before the tap.
- During a release the old version of the app can fail for a moment after the migration drops
  the table.
