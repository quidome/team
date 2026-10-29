---
title: The message for the next game
type: feature
stage: implement
closure: review build + accept   # from the epic; /design may only raise it
created: 2026-10-29
---

# The message for the next game

Story C of the epic [Duties](../2026-10-18-duties/). The coordinator opens "Message for the next
game" from a new part "Messages" on the start page and copies one Dutch text for the chat of the
team and the parents: the team's next game (when, where, home or away, the drivers) and the table
and referee places of the team in the coming week, with open places as "nog N open". The
functional spec is the epic's (`functional.md`, story C); the architecture is in the epic's
`technical.md`. Stories A and B are released as release 1 (`v0.5.0`).

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-29 | design | created from story C of the epic. Readings of the spec confirmed by the coordinator (R1–R5 in `technical.md`); the text forms stay in the core code (no templates); step 1 (the clipboard) is checked with browser tests only, the phone after step 4 |
| 2026-10-29 | implement | technical.md approved: one story of 4 steps (the clipboard with browser tests; the text in the core with exact-text tests; the route and the start page flag; the page and "Messages" on the start page); closure review build + accept |
