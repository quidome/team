# 0016 – An open change keeps the game as last seen

**Status**: valid
**Date**: 2026-10-13
**Change**: [2026-10-13-the-list-of-changes](../changes/done/2026-10-13-the-list-of-changes/)

## Context
From story 4 the tool keeps each change to a coming game in the schedule until the coordinator
has seen it, so a drive or a duty can be fixed in time. The functional spec asks for one change
for each game, "from what the coordinator last saw to what is true now", and no history. A game
can change twice before the coordinator looks, or move back. Readings run on a tap and (story 5)
each night, so a reading can change a change while the page with the list is open. Decision 0012
left open whether a change of club or team removes the unseen changes; the spec says it does.
Later changes (duties, messages) will build on these changes.

## Decision
- Table `changes`: one row for each game (club, season, the source's game id) with a change not seen yet. The row keeps the game as the coordinator last saw it (`NULL` for a new game), not the kind or the new values.
- The kind of change is worked out from that copy and the stored game now (`team_core::change_between`). A reading uses the same rule for its result line (`compare`). After a reading, `team_core::update_open_changes` decides for each game: insert (the stored game before the reading becomes the last-seen copy), a new version (the game changed again), or delete (the game is as last seen again, or past).
- Only games of today or later (Dutch date, as last seen or now) give a change; past games need no action (coordinator, 2026-10-13).
- A change is a change of the team when the team plays the game now or did when last seen, so its team look and filter stay when another club team takes the team's game.
- "Seen" deletes the row; there is no history. Each row has a version from one sequence (`change_versions`), also after a row was deleted and made again. "Seen" names the version it showed; a row with another version stays.
- One lock for all writers: a reading, the save of the choice and "seen" take the same advisory lock (`db::lock_readings`), so they run one after the other.
- The save of another club or team, or the first choice of a season (`team_core::same_choice` is false), removes all open changes in the same transaction. The next reading is a first reading and keeps none.

This answers the open point of 0012 ("Whether unseen changes of the old choice are removed (story 4) is still open"): they are removed.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Keep the game as last seen, work out the kind | One rule for the reading and the list; merging and "moved back" need no extra rules; nothing stored can disagree with the game | The kind is worked out again at each request (a few rows) |
| Store the kind and the old and new values at each reading | The list reads the table as it is | Merging needs rules for each pair of kinds; stored values can disagree with the game |
| Keep every change as an event, merge when showing | Full history | The spec asks for no history; the most code |
| Version counted per row (1, 2, …) | Simple | A row deleted and made again starts at 1 again, so an old "seen" could hide a newer change (build review R1) |

## Consequences
- Duties can later mark which open changes touch a game with a duty, from the same rows.
- A new hall name in the source (same place number) gives every open change at that place a new version, so a "Seen" from a page loaded before that reading leaves the change open with nothing new to see; one more tap.
- The result line counts one reading; the list shows last seen to now. A game that moved and moved back shows "1 moved" in the result line and nothing in the list.
- A gone game shows crossed out in the schedule only while its change shows; a past game that leaves the game list is hidden at once. What that means for duties is open (idea in `docs/ideas.md`).
