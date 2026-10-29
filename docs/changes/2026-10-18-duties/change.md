---
title: Duties
type: feature
stage: implement
closure: review accept   # epic: each story has its own closure (review build + accept); a fresh agent checks the whole result after story D
created: 2026-10-18
---

# Duties

The coordinator records the duty places of the team (drivers for its away games; its jury and
referee places at home games of other club teams, from the club's list), the player who does
each one, and after the game who really did it; the duty totals show that the work is shared.
From that the tool makes the message for the next game and the message for the referee
committee. An epic of four stories (see `idea.md`).

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-18 | idea | created from `/vision duties` (priority 3 and 4 of the vision; journeys 3, 4 and 5) |
| 2026-10-18 | discover | idea ready for discovery: an epic of four stories (A places, B who does it and the totals, C the message for the next game, D the message for the referee committee); answers of the coordinator in `idea.md`; the word "duty place" added |
| 2026-10-18 | design | functional.md approved: four stories (A duty places, B who does each duty place and the totals, C the message for the next game, D the message for the referee committee); spec review: all four "serves – note", notes S1–S9 answered (S1, S2, S4, S7 by the coordinator) |
| 2026-10-18 | implement | technical.md approved: an epic of four stories (A duty places, B who does each duty place and the totals — release 1 with A —, C the message for the next game, D the message for the referee committee), each with closure review build + accept; epic closure review accept; another team mid-season keeps the old duties (coordinator) |
| 2026-10-25 | discover | back to discover (from the design of story A): the screens of all four stories change. First the landing page and season overview (backlog item 1); then this spec is changed for them: no "Duties" page as the place for everything, open items as reminders, "handled" for possible duty games, labels in the overview. |
| 2026-10-25 | design | functional.md approved again (the rules stay, the screens change: the page of a game, the duty labels on "Season", duty open items for 4 weeks (drivers: this week and next), "Last" and "Next" with games with duties, "Messages" on the start page, a page "Duties" with the totals); spec review: A–C "serves – note", D "serves", notes S10–S17 answered (S10, S13, S14, S16 by the coordinator); technical.md must be changed for the new screens |
| 2026-10-25 | implement | technical.md approved again for the new screens: the same tables and core rules, a row with 0 places means "handled", the places travel on the answers of "Season" and the start page, two new open items, the page of a game, a page "Duties" with the totals, "Messages" on the start page; four stories, each with closure review build + accept; epic closure review accept |
| 2026-10-25 | discover | back to discover (from the reviews of story A): the referee committee gives the duty places of each home game of the club (referee 1 and 2, table 1 and 2) to club teams; the coordinator copies that schedule with pull-down lists and marks each game as ok (`/vision`, `idea.md` 2026-10-25). "Jury" becomes "table". Drivers stay. |
| 2026-10-25 | design | functional.md approved again: the 4 places of each home game of the club (referee 1 and 2, table 1 and 2) given to club teams with pull-down lists, saved at once; "Schedule entered" (check 1) and open places of the team (check 2); "places of the team" includes drivers; "jury" is "table"; spec review: A–C "serves – note", D "serves", notes S18–S23 answered by the coordinator; technical.md must be changed |
| 2026-10-25 | implement | technical.md approved again: the duties belong to the game (4 places of each home game of the club given to club teams, in `duty_teams`; "Schedule entered" a time on `game_duties`; drivers for the team); club teams from the stored games of the season; a row with 0 and 0 no longer means "handled"; the same four stories, each with closure review build + accept; epic closure review accept. The first build of story A is kept on the local branch `duty-places-v1` |
| 2026-10-27 | implement | story A, duty places, closed (`changes/done/2026-10-18-duty-places/`); for story B: R3, R4 and A1 of its reviews (keep a past game with filled driver places on a proposed number, a race test that fails without the row lock, whether a past swapped game counts its places) |
| 2026-10-27 | implement | A1 answered by the coordinator: what the coordinator stored is never changed by a reading of the game list. A past game counts what is stored (a filled place is done by its player unless corrected; "Nobody" counts for nobody), also after a later swap, a removal from the game list or another team chosen. A coming game that swaps or leaves the game list does not count its duties (as now). Agreed for the design of story B (a change of `functional.md`, to approve there): the list of changes also keeps a change to a past game with something stored (a place that names the team, drivers, players on places), with "has duties", until it is seen; it does not drop off when its date has passed. No separate conflict resolver and no separate open item: "Seen" means checked, a correction on the page of the game changes what counts |
| 2026-10-27 | implement | story B, who does each duty place and the totals: technical.md approved (`changes/2026-10-18-who-does-each-duty/`), 8 steps; functional.md changed with the coordinator (facts of 2026-10-27: A1, changes to past games with duties, clashes, the totals of the game's season) |
| 2026-10-28 | implement | story B, who does each duty place and the totals, closed (`changes/done/2026-10-18-who-does-each-duty/`, decision 0023); A4 of its accept review in `docs/ideas.md`. Next: release 1 (A and B), then story C |
| 2026-10-29 | implement | release 1 (A and B) is `v0.5.0`; story C, the message for the next game: technical.md approved (`changes/2026-10-29-next-game-message/`), 4 steps |
| 2026-10-29 | implement | story C, the message for the next game, closed (`changes/done/2026-10-29-next-game-message/`, decision 0024); A8 of its accept review in `docs/ideas.md`, A7 ("Copy" on the phone) in `docs/backlog.md`. Next: story D, the message for the referee committee |
