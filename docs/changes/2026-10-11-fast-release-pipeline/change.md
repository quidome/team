---
title: Fast release pipeline
type: technical
stage: implement
closure: checks      # the pull request and main runs are the test; the base review follows
created: 2026-10-11
---

# Fast release pipeline

Make the GitHub pipeline fast from the start: separate stages for checking, building, publishing
and releasing; a release only renames the image that `main` already tested.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-11 | design | created after measuring the pipeline of the base (7–20 min per push to main, ~12 min per tag) |
| 2026-10-11 | implement | technical spec approved: 4 steps, decision 0009 replaces 0008 |
