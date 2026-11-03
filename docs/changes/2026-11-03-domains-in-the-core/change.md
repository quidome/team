---
title: Domains in the core
type: technical
stage: implement
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
