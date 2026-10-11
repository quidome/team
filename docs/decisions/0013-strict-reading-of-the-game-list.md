# 0013 – Strict reading of the game list source

**Status**: valid
**Date**: 2026-10-11
**Change**: [2026-10-11-choose-club-and-team](../changes/done/2026-10-11-choose-club-and-team/)

## Context
The tool reads the club list and the games of a club from the game list source
(`db.basketball.nl`, the same data as `www.basketballstats.nl`). The source appears to be run by
a private person on the federation's data. It has no published agreement and no format promise;
the format can change without notice. The functional spec says the tool never shows wrong games
without a warning. Two facts found on 2026-10-11: the club name field inside a game holds team
names for some clubs, and an unknown club number gives an empty game list, the same as an
empty season.

## Decision
- All knowledge of the source's fields stays in `crates/server/src/source/`. Every answer is parsed field by field into the core types. A field that is missing, of another type, or not understood makes the whole reading fail with an error that names the club or game and the field (for example `game 1073597, field tijd: missing`). Nothing is saved after a failed reading.
- Fields the tool does not use (scores, results, logos, counts) are not read, so a change there cannot make a reading fail, and "0 - 0" can never become a result.
- Each game must be of the season that was asked for.
- Club names come from the club list only, never from a game.
- A club number is checked in the club list before its games are read.
- The source address is the setting `GAME_LIST_URL` (default `https://db.basketball.nl`). Requests have a 10-second timeout and the User-Agent `team-coordinator/<version> (+<ORIGIN>)`. The reader logs club, season and error kind on a failure.
- The API answers 502 `source_unavailable` (no answer or an error answer) or 502 `source_format` (data not understood).

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Field by field, fail on anything not understood | A format change is noticed at once and names the field | A harmless change in a used field also stops readings |
| `serde` structs with defaults | Less code | Errors do not name the field well; defaults can hide a format change |
| Read what can be read, skip bad games | Readings keep working | Can show a schedule with games missing and no warning |

## Consequences
- Stories 3 and 5 read games through this reader and follow the same rule.
- A format change of the source needs a code change; the test of the live source (`cargo test -- --ignored`) shows it.
- The test data in `crates/server/tests/data/game-list/` is the real answer of 2026-10-11.
