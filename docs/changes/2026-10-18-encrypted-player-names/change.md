---
title: Player names stored encrypted
type: technical
stage: close
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
| 2026-10-18 | implement | Release 1 (steps 1–3) merged as PR #13 before the reviews ran; the cluster was not restarted, and a fresh backup was made (`team-20261018T102433Z.dump.age`). Build review (R1–R6, no blocker) and accept review (accepted with notes, A1–A7): R1/A1 (a rename by the old pod during the update stays plain) and R2 fixed on `fix/encrypted-names-review`; the others answered. Closure stays review build + accept. |
| 2026-10-18 | implement | Release 1 live at 11:29 UTC: the 14 players in the database `team` encrypted, nothing lost, names right in the app. On the way: since 08:09 UTC Vault `DATABASE_URL` named the empty database `team_v2` (cause not found yet), so the app showed an empty tool; set back to `team` by the coordinator. Next: tag, then step 4 (release 2) after a nightly dump with encrypted names. |
| 2026-10-18 | implement | Step 4 built; backup with only encrypted names; release 2 merged (PR #15), live at 11:58 UTC, tagged v0.3.2; release steps 6 (no name in the players table file; WAL and `sessions` noted) and 7 (laptop dumps deleted) done. /close stopped: step 4 had no review. Build review of step 4 (R7-R11, no blocker): R7 (a backup from before v0.3.1 needs the v0.3.1 migration image once) documented in `README.md`; the others answered. |
| 2026-10-18 | close | Step 4 (release 2) gets the build review only: the coordinator chose not to run the accept review for step 4 (the accept review of release 1 covered the intent; the cluster check of release 2 showed the names right). R7 also in `gitops-lab` (PR #194). technical.md done. Next: `/close docs/changes/2026-10-18-encrypted-player-names`. |
