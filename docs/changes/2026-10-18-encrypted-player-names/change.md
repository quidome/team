---
title: Player names stored encrypted
type: technical
stage: implement
closure: review build + accept   # security, and the one-time encryption of the only copy of the names cannot be undone; the accept review checks that the coordinator sees no change and no plain name stays
created: 2026-10-18
---

# Player names stored encrypted

Store the first and last names of the players encrypted in the database (AES-256-GCM in the app,
the key from Vault and also kept outside the platform), so a copy of the database or a backup
does not show who the players are. Sorting and the same-name warning move from SQL to the core;
the existing players are encrypted once. Backlog item from the coordinator, 2026-10-18; the nightly
backup (built in `gitops-lab`) came first.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-18 | design | created from `/design technical: store the player names encrypted in the database` (the full request is in `docs/backlog.md`) |
| 2026-10-18 | implement | technical.md approved: 4 steps in two releases (steps 1–3 release 1, step 4 release 2), with release steps for the operator (the key in the password manager and Vault, the `gitops-lab` change, a fresh backup); closure review build + accept. Answers: players only, the one-time encryption in `team migrate`, two releases, laptop dumps deleted after release 2 |
