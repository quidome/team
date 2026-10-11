---
title: Fast release pipeline
type: technical
stage: done
closure: review build   # raised 2026-10-11: two surprises in step 3 (skipped publish, old workflow on an old tag)
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
| 2026-10-11 | implement | closure raised to review build: publish was skipped on main, and a tag on an old commit ran the old workflow |
| 2026-10-11 | done | closed: architecture.md has Build and release, patterns and technical debt; two idea lines |
