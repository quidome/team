# Ideas

Raw ideas, one line each. No judgement yet. `/vision idea: …` picks them up.
Add the date. Remove a line when it becomes a change, or mark it `(rejected: reason)`.

- 2026-09-27 Club duty allocation: record which team covers which jury and referee places at club home games (scenarios: features/duties/club-duty-allocation.feature).
- 2026-09-27 First-run setup: profile, club, schedule and teams in a few steps (scenarios: features/onboarding/first-run-setup.feature).
- 2026-09-27 More than one coordinator, each with their own teams (scenarios: features/coordinators/coordinator-teams.feature); 2026-10-11 the coordinator added: one coordinator may manage more than one team, and different teams in the next season. Story 2 keeps one row for each season so this needs no rewrite.
- 2026-09-20 Deferred in the first version: player accounts, trainer workflows, previous-season reports, notifications, offline use, direct WhatsApp integration (see docs/changes/done/2026-09-20-mvp/scope.md).
- 2026-10-10 Messages from templates, in a language of the coordinator's choice (the screens stay in English). For the messages change.
- 2026-10-11 Clean up old images in the registry: `build-…` and `sha-…` images are never removed (technical debt in docs/architecture.md).
- 2026-10-11 Automatic dependency updates (for example Renovate, as in gitops-lab) for Cargo, npm and the GitHub actions; then the actions can be pinned to exact commits at no extra cost.
- 2026-10-11 Read an earlier season again on request, for example to fill 2025-2026 after the start (left out of story 2, choose club and team).
- 2026-10-12 Test the held-back warning, "These games are really gone" and the failure line on the screen, not only at the API (story 3a, accept review A6). Since story 3b the browser tests replace API answers with `page.route`, so no changeable fake game list is needed.
- 2026-10-13 Duties and gone games: decide what a past game that leaves the game list means for its duties and the duty total (spec review of the deviations of story 4). Today such a game is hidden in the schedule at once.
- 2026-10-15 Start page with one list of coming events: trainings, games and duties together, with a small filter to hide cancelled items, and a way to go back through past events to correct them. After duties exist; needs a word for "a training, a game or a duty" (`/vision`). From the discovery of attendance at training.
- 2026-10-15 A longer login on the phone, so a training (a week apart) does not start with logging in; today a session lasts 8 hours (`SESSION_TTL_SECONDS`). From the second spec review of attendance at training (S11).
- 2026-10-17 A warning when the coordinator leaves the training page while a tap is still being sent or was not saved, so "Not saved. Try again." is not lost when the phone goes back in the pocket (accept review of story B of attendance at training, A3; the page has a "saving" mark and a 10 s limit).
