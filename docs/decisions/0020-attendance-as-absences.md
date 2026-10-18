# 0020 – Attendance as absences, one training per date

**Status**: valid
**Date**: 2026-10-17
**Change**: [2026-10-17-trainings-and-attendance](../changes/done/2026-10-17-trainings-and-attendance/) (story B of [attendance at training](../changes/done/2026-10-14-attendance-at-training/))

The number 0019 is not used. The header of migration `0005` names decision 0018 (players, roles
and weekly trainings) as "0019", the number planned before that record was written; an applied
migration is never edited, so 0019 was skipped rather than given to another decision.

## Context
Attendance at training is entered every week, in the hall, on a phone with one hand, often on a
weak network, and sometimes on two screens. It exists only in the tool. A training must exist only
when the coordinator opens a training day or adds one (decision 0018: weekly trainings only
suggest). Players join, change role and leave during the season, so who is "there" depends on the
roles, which can be corrected later. The coordinator must never see a mark as saved when it was
not, and must be able to correct every mistake.

## Decision
- **Absences only.** `absences` (training, player, reason) holds who was not there; the primary key is (training, player), so a player has at most one absence per training. "There" is not stored: a player is there when the training exists, the player has a role on its date, and there is no absence. Absences outside a player's roles are kept and do not count. Counts ("N absent", totals) are worked out from the stored rows at each request.
- **One training per date, guarded by the database.** `trainings.date` is unique. "Open" is `INSERT … ON CONFLICT (date) DO NOTHING` followed by a read, so it is safe to send twice and two screens give one training. "Add training" and "Change" check in the core first; a clash at the same moment fails on the unique index and answers `training_exists`. A training has no link to a weekly training: a change of a weekly training never touches it.
- **A training is opened by a tap.** The tap on a row of the list (or on the card) opens it; loading a page never writes. A page reached by its address shows "Not opened" with "Open training".
- **What deletes what.** Absences go with their training (`ON DELETE CASCADE`), so "Delete training" removes its attendance and nothing else deletes a training. A player with absences cannot be deleted (`ON DELETE RESTRICT`); the server turns the refused delete into `player_has_absences`, and the page does not offer "Delete player" then.
- **Taps on a slow network.** The last tap wins between two screens. On one screen, the taps of one player are sent one after the other: the row shows the new state at once with "saving…", and the next tap waits for the answer of the one before. A tap without an answer after 10 seconds counts as not saved: the row shows the stored state with "Not saved. Try again.". The answer of a reload, or of an action, that was sent before a tap does not undo that tap.
- **Marking needs no lock.** The player's role on the date is checked when a mark is saved; a role changed at the same moment can only leave an absence outside a role, which is kept and not counted.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Absences only; "there" follows from the data (chosen) | No rewrite when a role changes; deleting a player without absences loses nothing; totals always match the data | Counts are worked out at each request (cheap at about 15 players and 40 trainings a season) |
| Store "there" for each player at each training | Simple counting | Many rows to rewrite on every role change; a role corrected later leaves wrong rows |
| A unique date with `ON CONFLICT` (chosen) | The database guarantees one training per date; no lock to forget | A clash of "Add training" is found only at the insert (tested with an insert held on another connection) |
| An advisory lock for all training writes | One place for the rule | A write that forgets the lock can still make two trainings |
| Open on the tap (chosen) | A reload or a back button never makes a training; "Not opened" is seen before the tap | Opening a page by its address needs one more tap |
| Open on page load | One tap fewer from an address | A look at a past date makes it count with everyone there |
| One queue per player, with a time limit (chosen) | A late answer never overwrites a newer tap; taps of different players do not wait; a lost network is seen within 10 s | A tap that reaches the server after its limit is saved but shown as not saved until the page loads again |
| One queue for the whole page | Simple | One slow tap holds every other player |

## Consequences
- The totals of story C count, for each player, the trainings up to today that are not cancelled and fall inside one of the player's roles, minus the player's absences there; nothing extra has to be stored.
- Correcting a role ("Correct", "Add player" with an earlier date) can make a player count as there at trainings already entered; the tool says so ("This player now counts as there at N earlier trainings.") so the coordinator can mark absences.
- A training of an earlier season is reached by its address until story C adds the choice of season.
- No backup covers this data yet (the coordinator's choice, 2026-10-15; backlog).
