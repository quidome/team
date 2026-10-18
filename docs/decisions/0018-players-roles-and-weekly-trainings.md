# 0018 – Players, their roles in the team of a season, and weekly trainings

**Status**: valid
**Date**: 2026-10-17
**Change**: [2026-10-15-players-and-settings](../changes/done/2026-10-15-players-and-settings/) (story A of [attendance at training](../changes/done/2026-10-14-attendance-at-training/))

## Context
Attendance at training brings the first data that exists only in the tool: players, who is in the
team when, the training days and the reasons for an absence. Games can be read again from the game
list; this data cannot. Players join during a season, change from "only trains" to "plays games",
leave, come back, and stay for the next season, while the team gets a new name each season (the
team chosen from the game list, decision 0012). The coordinator must be able to correct every
mistake in the app. The first form of story A kept "training days valid from a date" for the
season; a mistake found later stayed in force for the past and was hidden (accept review A1).
Migration `0005` was written for that first form, which was never merged or released.

## Decision
- **The model** (coordinator, 2026-10-16). The club has the players: one club for the whole tool, so players have no stored link to a club. A team exists for one season and is the team chosen from the game list for that season. A player is the person (first name, optional last name, "hidden") and has no dates. A player is in the team of a season through a **role** ("plays games" or "only trains") with a start date and an end date (empty while it continues; at the latest 30 June). The roles of a player, one after the other, are the history.
- **Stored by season.** Roles (`player_roles.season_year`) and weekly trainings (`weekly_trainings.season_year`) are stored by season, not linked to the team choice. One team per season, so the season finds the team, and both can be entered before the team is chosen. Only roles and weekly trainings of the current season are changed; earlier seasons are shown, not changed.
- **Weekly trainings only suggest.** A weekly training is a day of the week with times and a window (a date until a date) inside one season. Its days are worked out each time, not stored. A change corrects it for its whole window; a real change of schedule is the end of one weekly training and a new one. A training exists only when it is opened (story B), and a later change of a weekly training never changes it.
- **Checks in the core, under a lock.** The roles of one player never overlap; two weekly trainings on the same day never overlap. The core checks both (`team_core::check_role`, `check_weekly_training`). Every write of a player's roles takes the row lock of the player (`players::lock`, `SELECT … FOR UPDATE`) in the same transaction as its read; every change of the weekly trainings takes an advisory lock ("team" "wktr"). No database constraint: an `EXCLUDE` constraint on date ranges needs `btree_gist` on the cluster.
- **Changes of the team from a date** (coordinator, 2026-10-16). "Change role" acts on the role that covers the date: it ends on the day before, and the new role keeps its planned end. "Remove from the team" from a date (the first day without the team) means: from that date the player has no roles in the season. The role on the day before ends then; roles that start on the date or later are **deleted**, after a confirmation that names them. Each of these saves is written whole or not at all.
- **"Undo" without stored state.** After "Remove from the team", the answer gives the player's roles of the season from before. "Undo" sends them back; under the player's lock the core applies the removal to them again (`team_core::undo_removal`). If the result is the stored roles, nothing else changed them, and the roles from before come back (with new ids; nothing refers to a role id). Else "Undo" is refused (`changed_meanwhile`). "Undo" lasts while the team page is open; later, "Correct" and "Add to the team" put a mistake right. There is no "Delete" for a role.
- **Migration `0005` was rewritten** before the first release of story A. Decision 0004 ("an applied file is never edited") means applied on the cluster; a file that never left a branch may still be rewritten, and local databases are made again. The header of `0005` names this decision as "0019", the number planned before Close; the applied file is not edited for that.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Roles by season, players without dates (chosen) | Joins, changes of role, leaves and returns are each a role; the history stays right; a new season starts empty and "Add all" brings last season's team back | More rows and rules than one date on a player |
| Dates on the player ("in the team since", "left on") | Simple for one season | A change of role or a return loses the history; every new season needs new dates |
| Players linked to the team choice of a season | The link is explicit | Nothing can be entered before the team is chosen; one team per season makes the link add nothing |
| Training days valid from a date, with a first and last date of the season (first form) | One form for the season | A mistake corrected later stays in force for the past, and no screen shows it (accept review A1) |
| Weekly trainings that only suggest (chosen) | A mistake is corrected for its whole window; opened trainings stay safe | A mistake that reached opened trainings is corrected on each training ("Change", story B) |
| Overlap checked in the core under a lock (chosen) | Works on the cluster as it is; one place for the rule, with unit tests | A write that skips the lock or the core is not stopped by the database |
| `EXCLUDE` constraints with `btree_gist` | The database stops every overlap | Needs an extension on the cluster's PostgreSQL |
| "Undo" with the roles from before, checked by the core (chosen) | No new table or column; a change from another screen is always seen | The core trusts the roles the page sends back (limited by `check_role`); right for one coordinator |
| A version number on each player | A simple comparison | A new column, and every write must remember to raise it |
| A stored undo record | The page sends only a code | A table and its cleanup, for a short "Undo" |

## Consequences
- Story B shows a player at a training, and counts the training for the player, only when the date is inside one of the player's roles (`team_core::role_on`). Absences outside a role are kept and do not count (the epic's functional spec).
- "Remove from the team" deletes roles. From story B, absences on the dates of a deleted role are kept and stop counting; "Undo" or a new role makes them count again.
- New writes of roles must take the player's lock and go through the core checks; the tests check each lock (a request waits while another connection holds it) and each save of more than one write (a test trigger makes the second write fail).
- No backup covers this data yet (the coordinator's choice, 2026-10-15; backlog).
