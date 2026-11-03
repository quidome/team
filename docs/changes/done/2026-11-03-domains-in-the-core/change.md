---
title: Domains in the core
type: technical
stage: done
closure: review build   # new check code and moved logic; the checks alone do not see a wrong placement
created: 2026-11-03
---

# Domains in the core

The product rules in `crates/core` get one folder for each domain (contribution, team, events,
game list, output, shared), with a test that fails when a domain uses a domain it may not use
(decision 0027). This follows the purpose in the vision of 2026-11-03: the tool shows what each
player gives to the team. No change for the coordinator.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-11-03 | design | created after the vision change of the purpose; the coordinator chose folders and a check of the direction over review only, one crate per domain, or only a record |
| 2026-11-03 | implement | technical.md approved: one story of 4 steps (the absent count to attendance; the stored game to the schedule; the folders; the test of the direction); closure review build |
| 2026-11-03 | implement | build review: R1 (the test missed `super::super` and `$crate::`) and R2 (the states of a training row stayed in events) fixed; step 1 changed after R2, chosen by the coordinator; R3 for Close |
| 2026-11-03 | implement | second build review (asked by the coordinator after the fix of R2): R4 (paths up from `mod.rs` and `crate as`) and R5 (no test that fails for the state of the last training) fixed; R6 for Close with R3 |
| 2026-11-03 | close | all 4 steps done, two build reviews answered (R1, R2, R4, R5 fixed; R3, R6 for Close); `just check` passes |
| 2026-11-03 | done | closed: domain map in `docs/architecture.md` (with `game_attendance` in contribution), lines on the start page, the list of a season and the page Season, a pattern and a debt line; `CLAUDE.md` names the domains; no new decision record |
