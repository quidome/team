# Players and team

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
The coordinator keeps the club's players and who is in the team when, so that attendance can be
marked without typing names, and the totals are fair to players who join, change role or leave
during the season. A player is the person: a name, with no dates. What has a start and an end is
the player's **role** in the team of a season: "Plays games" or "Only trains". Both roles come to
training. The roles of a player, one after the other, are the player's history.

Two pages: **Players** (every player the coordinator ever entered) and **Team** (the team of the
current season). The start page links to both, to [attendance](trainings-and-attendance.md) and to
the settings ("Attendance · Team · Players · Settings").

## Behaviour
### Players
- The page lists all players, sorted by first name, each with "Plays games" or "Only trains" when the player has a role today, else "Not in the team". A player whose role starts later shows "Not in the team" here, and is in the team on the team page.
- "Add player" adds a player with a first name (required) and a last name (optional). Names are saved without outer spaces; an empty last name is no last name.
- **Hidden** only keeps lists short; it is not a status. A hidden player is left out of the list on "Players" (unless "Show hidden players" is on; it then shows "· hidden"), the choices of "Add player" on the team page, and "Players of last season" (so "Add all" does not add it). Nothing else changes: a hidden player in the team stays in the team, the history and the totals on "Attendance" stay, and a hidden player under "Left the team" still has "Add to the team".
- A tap on a player opens the player: the name with "Save", the switch "Hidden", the history, and "Delete player" for a player with no absences.
- The history lists the roles of all seasons, newest first, for example "Plays games · from Sun 1 Nov" and "Only trains · Tue 1 Sept – Sat 31 Oct"; a role of an earlier season shows its season ("· 2025-2026"). A role of the current season has "Correct": the kind, "From" and "Until" (empty while the role continues), filled in with the stored values, with the same checks as a new role. For example a player added from 1 September who joined on 3 November, or added as "Plays games" by mistake. Roles of an earlier season cannot be corrected.
- "Delete player" asks first ("Delete Avery? This cannot be undone.") and deletes the player with all roles of all seasons, for example a player added by mistake. A player with absences cannot be deleted, only removed from the team or hidden: the button is not offered.
- When a role starts, or its start moves earlier ("Add player" from 1 September, "Add to the team", "Correct", "Add all"), while trainings on or after that date exist, the page says "This player now counts as there at 4 earlier trainings: Tue 6 Oct, Tue 13 Oct, Wed 21 Oct, Tue 27 Oct." ("there" is the default at a training), so the coordinator can mark absences. Cancelled trainings, and trainings where the player has an absence, are not listed. After "Add all", each line names its player.

### Team
- The heading shows the team chosen from the game list for the season and the season, for example "U14-1 · 2026-2027"; before a team is chosen, "Team not chosen yet" with a link to choose it ([choose club and team](choose-club-and-team.md)). Players can be added before the team is chosen.
- The page lists the players in the team today or later in the season, sorted by first name, each with "Plays games" or "Only trains"; then "Left the team": the players who had a role earlier in the season and have none today or later, each with "Left · last day Wed 14 Oct". A player after "Change role" stays in the team.
- **Add player**: a player from the list (not hidden and not in the team or under "Left the team"; last season's team first) or "New player" (a first name, and a last name if wanted), with a role and "In the team from". The date is filled with 1 September of the season, with a button "Today" next to it; any other date is typed or picked. So the first setup in October, with the trainings since 1 September, needs no typed date, and a newcomer in January takes one tap on "Today". The same first and last name as another player is allowed, with the warning "There is already a player with this name."
- A tap on a player in the team gives "Change role" and "Remove from the team"; a tap on a player under "Left the team" gives "Add to the team". The tap also shows the player's roles of the season.
- **Change role**: the new role and "From", the first day of the new role, filled with today. The role on that date ends on the day before; the new role starts on the date and keeps the planned end of the old role. A planned later role stays. So the history keeps both, for example "Only trains · Tue 1 Sept – Sat 31 Oct" and "Plays games · from Sun 1 Nov".
- **Remove from the team**: "Not in the team from" (the first day without the team), filled with today, with a button "Tomorrow" (for a player who still trains today). From that date the player has no roles in the season: the role on the day before ends on that day, and every role that starts on the date or later is removed. When the removal removes a role, the tool asks first and names it: "This also removes "Plays games · from Sun 1 Nov"." ("This removes …" when no role ends). A removal that only ends a role is saved at once.
  - A player with a role earlier in the season is under "Left the team" at once. For example on Thu 15 Oct from today: "Only trains · Tue 1 Sept – Wed 14 Oct", and "Left · last day Wed 14 Oct".
  - From tomorrow, the player stays in the team today and the row shows "Last day today". This is not shown on 30 June, when every role ends.
  - A player whose roles of the season all start on the date or later has nothing left in the season: in no group, and back in the choices of "Add player". So "Add all" in August, followed by "Remove from the team" from today for a player who did not come back, leaves nothing of that player in the season.
- **Undo**: after "Remove from the team", the player's row shows "Undo" until the coordinator leaves the team page. A player with nothing left in the season has a line below the team instead: "Blake · not in the team this season · Undo". "Undo" puts back the player's roles of the season exactly as they were. After that, "Correct" (end date empty) restores a role that was ended, and "Add to the team" a role that was removed; the history shows only the roles that are still there. There is no "Delete" for a role.
- **Add to the team**, under "Left the team": a role and "In the team from", filled with today. A new role from the date of return; the time between does not count.
- **New season.** A role ends at the latest at the end of its season (30 June); a new season starts with no players in the team. While no player of last season's team has a role in the season, the page shows "Players of last season" (last season's team, each with its role; not hidden) with "Add all". A newcomer added first does not make the group go. "Add all" asks "In the team from" (1 September, with "Today"); each player gets the same role as at the end of last season, from that date. Players of last season are still first in the choices of "Add player".

## Errors and limits
- "A first name is needed." – no first name.
- "This player has absences and cannot be deleted. Remove the player from the team, or hide the player." – a delete of a player with absences (the page does not offer it).
- "The end must be after the start." – the end of a role before its start ("Correct").
- "This date is outside the season." – a date of a role outside the current season; a role of an earlier season cannot be changed.
- "This player already has a role on that date." – two roles of one player that overlap.
- "The change must be after the start of the role. To change the role from its start, use "Correct" in the history." – "Change role" on the start of the role.
- "This player's role starts on Mon 2 Nov. To change it, use "Correct" in the history." – "Change role" on a date before a role that starts later.
- "The player already has this role." – "Change role" to the role the player has on that date.
- "This player is not in the team on that date." – "Change role" or "Remove from the team" on a date with no role and none later in the season (also "Remove from the team" from the day after the last day).
- "This player was changed in the meantime. Use "Correct" in the history." – "Undo" after another screen changed the player's roles.
- "This player was changed in the meantime. Please try again." – a confirmed removal whose roles changed on another screen before the second tap.
- "This no longer exists. Load the page again." – a player deleted on another screen.
- Players and roles only of the current season can be changed; one team per season; one coordinator. Two screens: the last save wins, and every check uses the stored roles.
- Player names never go to the log (ids only).
- The data exists only in the tool, and there is no backup yet.

## Interface
- Screens: `/players`, `/players/[id]`, `/team`; the links on the start page (`/`).
- Server addresses (all need a login): `GET /api/players` (all players with the role today), `POST /api/players` (add, with an optional role), `GET /api/players/{id}` (the player with the history), `PUT /api/players/{id}` (name and hidden), `DELETE /api/players/{id}`, `PUT /api/players/{id}/roles/{role}` ("Correct"), `GET /api/team` (the team page), `POST /api/players/{id}/roles` ("Add player" from the list, "Add to the team"), `POST /api/players/{id}/role-change` ("Change role"), `POST /api/players/{id}/team-leave` ("Remove from the team", with the confirmed roles), `POST /api/players/{id}/team-undo` ("Undo"), `POST /api/players/carry` ("Add all", with a date).
