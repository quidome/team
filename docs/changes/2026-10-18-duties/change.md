---
title: Duties
type: feature
stage: design
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
