---
title: The base
type: technical
stage: done
closure: review build   # login, sessions and publishing: a fresh review of the diff
created: 2026-10-10
---

# The base

Story 1 of the epic [Games from the game list](../2026-10-10-games-from-game-list/). Replace the
old app with the base of the new product: Rust server, PostgreSQL with migrations, login through
Pocket ID, and a Svelte front end with its look, deployed to the existing cluster contract.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-10 | design | created from story 1 of the epic |
| 2026-10-10 | implement | technical spec approved: 5 steps (step 0 by the coordinator), decisions 0007 and 0008 |
| 2026-10-10 | implement | steps 0–3 done; branch feat/the-base merged into main |
| 2026-10-11 | implement | step 4 done: look approved on the cluster, v0.1.0 released |
| 2026-10-11 | implement | build review: 6 findings, all fixed (R5: keys fetched again on a signature failure, chosen by the coordinator) |
| 2026-10-11 | done | closed: architecture (layers, login, request rules, patterns, debt), product doc login.md, decision 0011, vision "where we are" |
