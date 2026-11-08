---
title: Not played
type: feature
stage: close
closure: review build + accept   # from the epic; story E changes released rules of duties
created: 2026-11-08
---

# Not played

Story E of the epic [Attendance at games](../2026-10-31-attendance-at-games/). The coordinator marks
a game "Not played" with one tap. Its duties and attendance then stop counting, and it gives no
warning, no open item and no line in the messages. Nothing stored is deleted, and "Undo" makes it
count again. From the start time, the switch "The drivers drove" lets the drives of a not played game
count.

Spec: story E in [functional.md](../2026-10-31-attendance-at-games/functional.md) of the epic.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-11-08 | design | created from `/design story E of attendance at games`; story E gets the switch "The drivers drove" and "can still be changed" (coordinator); the rules for "Checked" stay in story C; each domain owns its tables, treated as if it were a service (coordinator) |
| 2026-11-08 | implement | technical.md approved: 4 steps (the mark stored; duties stop counting; warnings, open items and messages; the page and the rows); deviations D1 (own table in events, switch in `game_duties`), D2 (the switch, "can still be changed"), D3 (the button on every game); spec review of the deviations: all serve, N1 fixed (the switch from the start time); decision records 0030 and 0031 at close |
| 2026-11-08 | close | 4 steps built; review accept: accepted with notes (A1 fixed with R2; A5 to story D, A6 to story C, A12 to the product documents); review build: R1–R7 fixed (start without a start time, the switch counts in the totals only, no driver note, a test that could not fail, a race test, the move note in the core, a test that leaned on another), R8 for close; `just check` passes |
