# Idea: Games from the game list

**Status**: ready for discovery
**Vision fit**: Priority 1 in the vision ("Games from the club's game list"). Follows the principle "The schedule comes from the game list, not from typing" and user journey 1 ("Games come in").
**Date**: 2026-10-10

## The problem
The coordinator keeps the schedule by hand in a Google Sheet. Typing games costs time and can
be wrong. Games also move during the season, and the coordinator must notice that, because a
moved game can break a drive or a jury or referee duty that is already arranged. Everything else
the tool does (attendance, duties, messages) depends on correct games.

## The idea
The tool reads all games of the club from the federation's game list: the open data service of
the Nederlandse Basketball Bond (the Dutch basketball federation), described at
https://www.basketballstats.nl/help/koppelingen. It reads all club games, not only the team's
games, because the club's home games are where our players can get jury and referee duties.

- The coordinator can start a reading at any time. The tool also reads on a schedule.
- The tool does not change games. The game list is the truth. (Expected by the coordinator, not yet confirmed; see open questions.)
- There is no approval step. Changes from the game list go in at once.
- Each change (for example "moved from Saturday 16:00 to Sunday 15:00") stays visible until the coordinator marks it as seen.
- When duties exist (a later change), a change to a game that has duties is shown as more important.
- No phone notifications in the first version. The coordinator opens the tool every week for attendance. Add notifications only if real use shows that changes are missed.

Suggested order of steps, for Discover to confirm: reading on demand first, then reading on a
schedule together with the list of changes since the coordinator last looked.

## Why now
It is priority 1, and the rest of the product depends on it. It is the first step to replace the
sheet.

## How it fits
- Vision: matches journey 1, priority 1 and the principle on the schedule. No vision change needed.
- Words: "game list" in `docs/language.md` is this data service. No new words.
- History: the old app had a version of this (behavior described in `features/club/club-schedule-sync.feature`; rules for matching, updating and preview on the parked branch `feat/club-schedule-sync`, commit 3f28ba4). That version had a preview-before-apply step; this idea drops it on purpose (see above). Use the old work as reference only.

## User view
For the coordinator only. Two moments matter:
- **The first time**: the whole season of the club appears without typing.
- **When a game moves**: the coordinator sees what changed, checks what it breaks, and marks it as seen.

Approval would feel awkward: saying "no" to a moved game does not stop the game from moving.

## Open questions
- What data does the game list give exactly (fields of a game, identifiers, formats), and which form of the service (JSON, XML, CSV, calendar file) fits best?
- The season half: the coordinator sees games for the whole year for one team, but only until the end of the calendar year for the other club teams. Teams appear to get new identifiers in the second half. Check this in last season's data. Must the first version handle it, or can it wait until January?
- Does the coordinator ever need to correct or add to a game (for example a game with no start time, or a game that the club moved before the federation updated its data)? This decides if "the tool never changes a game" holds.
- What happens when a game disappears from the game list?
- How often does the scheduled reading run, and what does the coordinator see when a reading fails?
- How does the coordinator tell the tool which club is theirs and which team is theirs?
