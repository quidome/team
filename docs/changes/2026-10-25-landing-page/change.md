---
title: The start page as a landing page
type: feature
stage: implement
closure: review build + accept   # replaces the page used most; four rules meet in the open items
created: 2026-10-25
---

# The start page as a landing page

Story B of the epic [Start page and season](../2026-10-25-start-page-and-season/). The start page
shows "Open items" (problems with the game list, changes not seen, training days not opened,
trainings not checked), "Last" (the last training before today) and "Next" (the games of the team
and the trainings of this week and next week), and the links "Season · Attendance · Team · Players ·
Settings". The schedule, the card of the next training, the "Game list" card, the line of changes
and the view "Last season" leave the start page. The functional spec is the epic's
(`functional.md`, story B); the architecture is in the epic's `technical.md`.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-25 | design | created from story B of the epic |
| 2026-10-25 | implement | technical.md approved: a story of 6 steps (core `start_page`; shared reads; `GET /api/start`; the start page; the old browser tests; removing the old parts); closure review build + accept |
