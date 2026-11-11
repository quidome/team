---
title: A quieter start page
type: review
stage: close
closure: review build + accept
created: 2026-11-10
---

# A quieter start page

The coordinator finds the start page overwhelming: it shows everything that is true (the last
events, two weeks of coming events, done things, rare buttons and seven links), so the two answers
the page exists for, "does something need me?" and "what comes next?", are lost. This review makes
the page show less, and show it more quietly.

## Starting point (from the conversation of 2026-11-10)

The coordinator looked at a screenshot of the page with real data, and at mockups on a design
canvas (https://claude.ai/artifact/9mH5jgbtjyEJwLyXUoQTJ1, private to the coordinator; the chosen
mockups are "B · One quiet list", "E · Combined" and "F3 · Busy Saturday, grouped by date"). The
coordinator chose:

1. A quiet list without cards (mockup B): events with nothing left to do are lighter.
2. The open items stay at the top, one row each with an orange dot; a tap opens the place to act.
3. Past events show only when they are not checked, and only as an open item: the part "Last" goes.
4. The kind of event shows only for the games: a game of the team has a soft orange background and
   an orange bar, a game with places of the team (not a game of the team) has a thin blue bar, a
   training has no mark (mockup F3).
5. The events are grouped by date: the date is a small heading, the rows under it show their
   start times.
6. "Message of the week" stays on the start page as one quiet link. The committee message moves
   to the page "Duties", next to the duty overviews.
7. Rare things leave the first view: "Change" (the choice of club and team) and the links Team,
   Players and Settings (under "More").

The open points and the two spec reviews changed some of these choices (the duty games of the
last date with duties stay; the committee message stays on the start page in weeks with places).
`functional.md` is the approved version.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-11-10 | discover | created as a review of the start page; starting point from the coordinator's choices on the mockups |
| 2026-11-10 | discover | functional spec written; two spec reviews (serves – note), their notes answered with the coordinator; spec approved |
| 2026-11-10 | design | technical spec approved: one story in 4 steps (core rules, new places, start page, check of the look); closure review build + accept (a rule moves out of the messages, the start answer changes shape, a part of decision 0026 is replaced; the reason is a feeling that only a fresh look can check) |
| 2026-11-11 | implement | 4 steps built on the branch quieter-start-page; build and accept reviews: A1/R1 (the last duty games always lighter, as the spec says) fixed with the coordinator, R2–R4 fixed, R5 for Close |
