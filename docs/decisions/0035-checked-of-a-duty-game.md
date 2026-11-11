# 0035 – "Checked" of a duty game

**Status**: valid
**Date**: 2026-11-11
**Change**: [2026-11-11-checked-for-a-duty-game](../changes/done/2026-11-11-checked-for-a-duty-game/) (story 2 of [Checked for every event](../changes/2026-11-11-checked-for-every-event/))

## Context
"Checked" means that an event is done in the administration and no longer asks for attention
(word list, 2026-11-11). A training and a team game had it; a duty game (a home game of another
club team where the team has table or referee places) did not, so it could never leave the
attention. The check of a team game (decision 0032) sits in the attendance of the game and makes
each player with no answer absent; a duty game has no attendance. When another team is chosen, a
duty game can become a team game, and the other way round.

## Decision
- **The duty check is stored with the duties of the game:** two columns of `game_duties`
  (`checked_for`, the date of the game when it was checked; `checked_at`, the first moment),
  migration `0017_duty_checks.sql`, written under the row lock of the game in duties (decision
  0023), as "Schedule entered" and "The drives count". Not in `game_attendance`.
- **One rule for a mark that holds for a date** (decision 0029): `team_core::mark_holds` in the
  domain shared, used by "Not played", the check of a team game and the duty check.
- **Each check holds only for its own kind of game:** `team_core::check_kind` (a team game:
  `Team`; another game with table or referee places of the team, also one kept for its duties:
  `Duty`; else none) and `team_core::game_checked` (the check of its own kind holds for its date;
  a not played duty game is never checked). The page of a game (`duty_checked`), "Season" and the
  start page use this one rule.
- **One address for both kinds:** `PUT` and `DELETE /api/games/{season}/{id}/checked` choose by
  `check_kind`: a team game as in decision 0032; a duty game stores only the duty check (`settled`
  0), from its start time, not on a game marked "Not played" (`duty_game_not_played`); another game
  is `not_checkable` (was `not_a_team_game`). The page sends the kind it shows (`?kind=team` or
  `?kind=duty`); a mismatch is refused with `game_changed`, so a page that is out of date never
  runs the check of the other kind.
- **Only the attention changes:** a filled place still counts as done from the start time.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Two columns of `game_duties` (chosen) | The owner and the lock of the duties of a game; as the other marks of duties | – |
| A new table `duty_checks` (the first design) | A table for one fact | One more table with the same owner, key and lock |
| A column in `game_attendance` | One place for "checked" | The attendance of a team game; its check makes players absent; another domain's table (0030) |
| A separate address `.../duty-checked` | Each kind its own action | The page and the panel would choose the action; the server knows the kind |
| No `?kind=` | A simpler address | A page that is out of date could run the team check on a former duty game and make every player absent |

## Consequences
- A stored check of the other kind stays in the database but has no effect, and holds again when
  the game becomes that kind again.
- The open item of duty games that are not checked (story 3) reads the same rule.
- The duty check belongs to the game, not to the team: when another team with places at the same
  game is chosen, the check still holds (as the check of a team game).
