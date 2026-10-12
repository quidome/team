# 0015 – Readings of the game list are recorded results

**Status**: valid
**Date**: 2026-10-12
**Change**: [2026-10-11-read-on-demand](../changes/done/2026-10-11-read-on-demand/)

## Context
From story 3a the tool reads the club's game list when the coordinator asks, and later (story 5)
each night. The screen must show the failure of the last reading together with the time of the
last good one, so a failure is a result to keep, not only an error to answer. Decision 0013 said
the API answers 502 on a failure and that nothing is saved after a failed reading. Decision 0012
planned to use `chosen_at` to find the first reading after a choice; the build review showed that
every save of the choice renews `chosen_at`, also when nothing changed, so a real change would go
uncounted. A reading that waits for the lock may hold an older answer of the source than the one
already stored.

## Decision
- Every reading is a row in `readings`: good, failed (`source_unavailable` or `source_format`) or
  held back (`empty` or `far_fewer`). `POST /api/readings` answers 200 with the reading, also when
  it failed or was held back. A failed or held-back reading changes no games.
- A reading uses the stored choice of club and team. It does not check the club list again: the
  club number was checked against the club list when the choice was saved (decision 0013).
- Each reading stores the team it was read for. A reading is the first one after a choice when
  the last good reading of the same club and season had another team, or there is none
  (`team_core::is_first_reading`). The first reading counts nothing.
- "These games are really gone" is a reading in its own mode: under the lock, the confirmed
  reading must still be the newest reading that did not fail, and the new reading is accepted only
  with at least as many games as the warning showed (`team_core::guard_after_confirm`); otherwise
  it is held back again.
- Under the lock, a reading that finds a good reading of the same club and season that started
  later writes nothing and answers with that reading, so an older answer never replaces a newer
  one.

This replaces, for readings, these parts of earlier records:
- 0013: "The API answers 502 `source_unavailable` … or 502 `source_format`" and "Nothing is saved
  after a failed reading" (no games are saved; the reading itself is). The choice of club and team
  keeps both rules.
- 0012: "`chosen_at` is for the later stories: the first reading after a choice records no
  changes". Whether unseen changes of the old choice are removed (story 4) is still open.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| A failure is a recorded reading, answered with 200 | The screen shows the failure and the last good reading from one place; the nightly reading leaves a trace | Two ways to fail: an HTTP error and a failed reading |
| A failure is only an HTTP 502 (as 0013) | One error path | The nightly reading's failure is lost; the screen cannot show it later |
| First reading from `chosen_at` (as 0012 planned) | No extra column | Saving the same choice again hides real changes |
| First reading from the team of the last good reading | Changes only when the team really changes | One more column in `readings` |
| Check the club list at each reading | A wrong club number is found at once | One more request each reading; the number was checked when saved |

## Consequences
- An unknown club gives an empty game list. The guard holds that back within a season, but a
  first reading of a season accepts it (as an empty summer season). This is accepted because the
  club number comes only from a saved, checked choice.
- `GET /api/readings/latest` gives the newest reading, the last good one, and the held-back
  reading to confirm (the newest that did not fail, when held back), so a failed tap does not
  hide the warning.
- Story 5 (the nightly reading) uses the same function and the same lock.
