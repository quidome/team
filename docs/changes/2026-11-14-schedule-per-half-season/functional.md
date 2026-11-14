# Functional spec: Schedule not entered for each half season

**Status**: approved
**Change**: docs/changes/2026-11-14-schedule-per-half-season
**Date**: 2026-11-14

## Context
The referee committee of the club gives the table and referee places of the club's home games to
the club teams. It makes that schedule once for each half season (1 July – 31 December, 1 January
– 30 June), for the home games of all club teams, and it can do so only when the club teams have
their games for that half. The coordinator enters the committee's schedule in the tool and marks
each home game "Schedule entered".

Youth teams play a competition for each half season; the games of the second half come in the
game list only after the first half (in 2025-2026 the second half started on 17 January). Senior
teams play one competition for the whole season, so their games of the second half are known from
the start. The game list also does not come at once on 1 July: it is published weeks or months
into the season, and it is not known whether all teams of a club come at once or one by one.

Today the open item "schedule not entered" on the start page lists the home games of the club from
today until 27 days later whose schedule is not entered. That fails in two ways:
- **Too early.** Now only MSE 1, a senior team, has games after 1 January. From about 13 December
  its January home games give the reminder, while the committee cannot make a schedule from one
  team.
- **Too narrow.** When more teams have their games, the committee can start, and the coordinator
  wants to know, to ask the committee for its schedule. A reminder of only the next 27 days does
  not show that a whole half season waits for a schedule.

The coordinator's rule: when more than 2 club teams have their games for a half season, the
committee can start, and the coordinator wants to be aware of it to ask the committee. No one
knows how many club teams the half season will have in the end, so the tool cannot say when the
schedules are complete; it can only see which club teams got new games for the half.

"Got new games for the half" must leave out a senior team, whose one competition for the whole
season was known from September: its games after 1 January say nothing about the second half.
Every game in the game list names its competition, so the test is: **a club team counts for a half
season when one of its competitions has its first game in that half.** See "Evidence from the game
list" below.

**Serves the purpose**: core value 2 (share the duties fairly and easily): the reminder about the
committee's schedule shows when the coordinator can act on it (ask the committee, then enter its
schedule), and is silent before. This is also "Where we are going" in `vision.md`: attention only
when there is something to do.

## User stories

### Story 1: no reminder for a half season that is not ready
**As a** coordinator, **I want** no reminder "schedule not entered" for the home games of a half
season for which 2 club teams or fewer count, **so that** I am not asked for a schedule that
the referee committee cannot make yet.

#### Acceptance criteria
- [ ] A club team counts for a half season when one of its competitions has its first game in that half (Dutch date). The first game is the earliest game of that club team in that competition, home or away, that is still in the game list.
- [ ] A club team with one competition for the whole season (a senior team, first game in September) counts for the first half only. A club team with a new competition for the second half (the youth teams, and a senior team that plays halves, as VSE 1 in 2024-2025) counts for the second half.
- [ ] The club teams are the teams of the club with a game in the game list of that season that is still in it; the team of the coordinator is one of them. Two club teams in one competition count as two.
- [ ] A game marked "Not played" still counts as a game of its competition.
- [ ] In a half season for which 2 club teams or fewer count, no home game of the club of that half is in the open item "schedule not entered", also not a home game within 27 days.
- [ ] Example: the second half of 2026-2027 has games of MSE 1 only, in its competition that started on 12 September 2026, so no club team counts for the second half. On 20 December 2026, the home game of MSE 1 on 9 January 2027 (within 27 days) without a schedule is not in the open item.
- [ ] The rule is the same for both halves of a season.

#### Behaviour
The coordinator opens the start page in December. The open item "schedule not entered" lists only
the home games of the first half without a schedule, as before (if any). The January home games of
MSE 1 are not in it. Their rows on "Season" still say "Schedule not entered", as now.

#### Errors and edge cases
- A game that moves from one half season to the other counts for its new half from the next reading of the game list.
- A game that leaves the game list no longer counts for its team. When that makes a half season drop to 2 club teams or fewer, its home games leave the open item.
- A club team that withdrew from the competition but still has games in the game list counts as a club team. The coordinator cannot take its games off (outside this change).
- A home game that is near, in a half season that is not ready (the committee scheduled one team on its own): the tool gives no reminder. The row on "Season" still says "Schedule not entered".
- At the start of the season, before the game list has games of more than 2 club teams, no home game of the first half gives the reminder. Every competition has its first game in the first half, so in the first half every club team with games counts.
- A club team that keeps its competition, with the same number, for the second half (not seen in four seasons, see the evidence) does not count for the second half. The half season is then ready one club team later. The rule never makes a half ready too early because of it.
- The first game of a competition leaves the game list or moves: the earliest game that is still in the game list counts, from the next reading.

### Story 2: all home games of a half season that is ready
**As a** coordinator, **I want** the open item "schedule not entered" to list every coming home
game of a half season for which more than 2 club teams count, **so that** I know the committee
can make its schedule now, and I can ask the committee for it.

#### Acceptance criteria
- [ ] In a half season for which more than 2 club teams count, every home game of the club of that half from today whose schedule is not entered is in the open item, also when it is more than 27 days away.
- [ ] As now, these home games are not in it: past games, games no longer in the game list, games that are no longer a home game of the club, and games marked "Not played".
- [ ] The text stays as now for the first half: "Home game Sat 31 Oct: schedule not entered" for one, "18 home games: schedule not entered" for more.
- [ ] When the item holds a home game of the second half, the text adds how many club teams are known for the second half: "18 home games: schedule not entered · second half: 3 of 4 teams known" ("Home game Sat 9 Jan: schedule not entered · second half: 3 of 4 teams known" for one).
- [ ] "Known" are the club teams that count for the second half (story 1). "Of 4" is the known club teams plus the club teams that have games in the first half but none in the second half: the tool takes such a club team as waiting for its second half. A club team that has games in both halves but does not count (a senior team with one competition for the season) is in neither number.
- [ ] Example, today's game list (14 November 2026): M16 1, M16 2, M18 1 and X14 1 have games only until December, MSE 1 has games into April, so the second half has 0 of 4 teams known. When three of the four have a competition that starts in January: "3 of 4 teams known".
- [ ] As now, the item and its list of games hold the same games; for one game a tap opens it, for more a tap opens the list below the item, in time order, each a tap to its page.
- [ ] When both half seasons are ready (for example in December, with home games of both halves without a schedule), the one item counts the home games of both.
- [ ] The item goes away by itself when every listed home game has its schedule entered, and comes back by itself when new home games without a schedule come in.
- [ ] Example: 3 club teams have a competition with its first game in the second half, and none of its 18 coming home games has a schedule, and one more club team has games only until December. The item shows "18 home games: schedule not entered · second half: 3 of 4 teams known".

#### Behaviour
The youth teams get their second half in the game list. When the third club team has a competition
that starts in the second half, the next reading puts every coming home game of the second half without a schedule in
the open item. The coordinator sees "18 home games: schedule not entered · second half: 3 of 4 teams known",
knows that one club team is still missing, and asks the committee for its schedule. When the schedule comes, the coordinator opens each home game from the list,
enters the places and marks "Schedule entered"; the number goes down, and the item goes away when
all are entered.

#### Errors and edge cases
- The item stays (orange, as every open item) while the coordinator waits for the committee's schedule, possibly for weeks. The coordinator accepts this: the item is the reminder to ask the committee, and then the list to enter its schedule (spec review S3).
- A club team that withdrew after the first half, or that has no second half, stays "waiting": the text says "3 of 5" for good. The coordinator knows why; the half is ready all the same.
- "Waiting" is what the tool sees, not proof: a club team with games only until December is taken as one that gets a second half, as in each of the four seasons seen.
- More teams get games after the coordinator entered the schedule: their new home games without a schedule bring the item back.
- A home game moves into a ready half season: it counts at the next reading, when its schedule is not entered.
- A half season that is ready drops to 2 club teams or fewer that count (games left the game list): its home games leave the open item; see story 1.

## Interface
- Screen: the start page (`/`), open item 4 "schedule not entered" ([start page](../../product/start-page.md)). Its place in the order of the open items and its list do not change. It changes which home games it holds, and with home games of the second half its text adds "· second half: N of M teams known".
- No other screen changes. The page "Season" and its label "Not entered", the page of a game and "Schedule entered" stay as they are.

Examples (the club teams are from the game list on 2026-11-14; the dates and numbers of home games are made up), on 15 December 2026, before the youth teams have their second half:

| Half season | Club teams that count | Home games without a schedule | In the open item |
|---|---|---|---|
| First half (until 31 Dec) | 5 (M16 1, M16 2, M18 1, MSE 1, X14 1) | 1 (19 Dec) | it |
| Second half (from 1 Jan) | 0 (MSE 1 has games, but its competition started in September) | 2 (9 Jan, within 27 days, and 16 Jan) | none |

Text: "Home game Sat 19 Dec: schedule not entered".

On 20 December 2026, after the game list has the second half of three youth teams:

| Half season | Club teams that count | Home games without a schedule | In the open item |
|---|---|---|---|
| First half | 5 | 0 | none |
| Second half | 3 (of 4: one youth team still has games only until December) | 18 (9 Jan – 24 Apr) | all 18 |

Text: "18 home games: schedule not entered · second half: 3 of 4 teams known".

## Consistency
- The rule only narrows (a half season that is not ready) or widens (no 27-day limit in a ready half) which home games the existing open item holds. The item, its list and its taps work as now; its text adds "· second half: N of M teams known" only with home games of the second half.
- "Which dates count" stays: only home games from today; the Dutch date.
- In July and August the open items also cover the season before; its home games are past, so they are never in this item.
- The open item "open places" (places of the team without a player, from today until 27 days later) does not change: a place gets a player only after the schedule is entered.

## What changes for existing users
No breaking changes. Today (14 November 2026) 5 club teams count for the first half, so the
item works as before for its home games, without the 27-day limit (the first half ends on 31
December, about 47 days away). The home games of the second half (MSE 1 only, no club team counts) give no reminder.

## Warnings from architect and engineer
- **[Steve]** A club team that withdrew but still has games in the game list counts. With a threshold of 2 of 5 or 6 club teams the effect is small; a club with 2 teams would never get the reminder. Acceptable for a tool for one coordinator.
- **[Steve]** The number "more than 2" is fixed, not a setting.
- **[Bruce]** Whether a half season is ready must follow the games as they are now, at each reading of the game list: a game that moves across 1 January, or leaves the game list, must change the item at once, with no state left from before.
- **[Bruce]** The count of the item and its list must stay equal in every case, also in July and August.
- **[Bruce]** The rule needs the competition of each game. The tool may not keep it today (not checked); the games read before this change must get it too, or the rule cannot see the competitions of the stored games.
- **[Bruce]** Competition numbers are not in order over time (see the evidence): the rule may use only the dates of the games, never the order of the numbers, and compares numbers only within one season.

## Evidence from the game list
The games of the club (club 37) in four seasons, read on 2026-11-14 (2023-2024 to 2026-2027):
- Every youth team had a new competition, with a new number and a first game in January, for the second half: 2023-2024 from 12 to 20 January, 2024-2025 from 11 January, 2025-2026 from 17 January. Also when it stayed in the same division (for example M16 1 in 2025-2026: 4th division in both halves, competitions 4088 and 4425).
- Senior teams: MSE 1 played one competition from September to April in all four seasons; VSE 1 also, except in 2024-2025, when it played a first and a second half.
- No competition number came back in another half or another season of this club. Two club teams shared one competition once (X14 1 and X14 2, first half of 2024-2025).
- The numbers do not go up over time (2025-2026 has lower numbers than 2024-2025).
- In 2026-2027, on 2026-11-14: 5 club teams have games (X12 1, in the list on 2026-10-11, is no longer in it); only MSE 1 has games after 1 January.

A team that keeps its competition number for the second half is possible but was not seen; that
several youth teams do so in one season is not likely. Then the half is ready one team later.

## Not in scope
- The label "Not entered" on "Season" and the rows "Schedule not entered" on "Season": unchanged (the change "Review of the page Season" decides about them).
- A list for each club team of its games in each half season: parked in `docs/ideas.md`. The item shows only the numbers.
- "second half: N of M teams known" for the first half: there is no half before it to compare with.
- A way to take off the games of a club team that withdrew (ideas list, 2026-11-01).
- The open item "open places".

## Open questions
None.

Answers to the second spec review (coordinator, 2026-11-14): S6 and S7 fixed in the documents; S8:
the text is "· second half: 3 of 4 teams known", and the word list allows "teams" for club teams
in a count on a screen. S9 and S11 are for Design.

Answer to spec review S1 (coordinator, 2026-11-14): a club team counts for a half season only
when one of its competitions has its first game in that half, so a senior team with one
competition for the season never makes the second half ready. Added on the same day: with home
games of the second half the item says how many club teams are known ("second half: 3 of 4 teams known").

## Spec review

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| 1 – no reminder for a half season that is not ready | Looks at the start page in December and must not be asked for a schedule that the committee cannot make yet | serves – note | Stops a reminder the coordinator cannot act on, as "Where we are going" asks. Note S1 (senior teams count in "more than 2") and S2 (the example cannot fail today). |
| 2 – all home games of a half season that is ready | Learns that the committee can start, asks for its schedule, then works through every home game of the half to enter it | serves – note | The full list without the 27-day limit is the walk list for entering the schedule in one go. Note S3 (the item stays orange for weeks while the coordinator waits for the committee). |
| Purpose line | – | serves | Core value 2 ("easily"): it cuts wrong reminders and gives one list for entering the schedule. This is supporting work for that value and stays small (one rule, no new screen). |

**Overall: serves – note.** Design can start. S1 needs the coordinator's answer first.

- **S1. Senior teams count toward "more than 2".** The idea asked whether all club teams count or only those with a half-season competition ("a senior team always has the second half, so it would count before any youth team is known"). The spec counts all club teams but gives no reason. In the 2025-2026 game list the club had two senior teams (MSE 1 and VSE 1, both with games in the second half). In such a season the second half is "ready" as soon as one youth team gets its second half: 2 senior teams plus 1 youth team out of 6 club teams. That is the early reminder this change is meant to stop. In 2026-2027 the reader's test copy has one senior team (MSE 1), so 2 youth teams are needed. Whether "more than 2 teams" in the coordinator's rule means youth teams or any club teams is a claim about the user's intent: not checked, ask the coordinator. Checked by a count of the club teams with games in each half in `crates/server/tests/data/game-list/games-37-2025-2026.json` and `games-37-2026-2027.json` (a script over `wedstrijden`, `thuis_club_id`/`uit_club_id` = 37).
- **S2. The 5 December date and the example in story 1.** In the 2026-10-11 copy, the first January home game of MSE 1 is on 9 January 2027. The 27-day window reaches it on 13 December, not on "about 5 December" (Context, `change.md` log). On 5 December no January game falls within 27 days, so the example acceptance criterion ("On 5 December 2026, the home games of MSE 1 in January 2027 ... are not in the open item") is also true under the current rule and cannot fail. The same goes for the first example table in Interface (9 and 16 Jan, on 5 December). A date from 13 December on, for example 20 December with the 9 January game, would test the rule. The deadline is less tight than stated, but the live game list may differ from the copy (not checked). Checked by a list of the club's coming home games from `games-37-2026-2027.json`.
- **S3. A ready half asks for attention while there is nothing to do.** Story 2 shows the item as soon as the third club team has games, so the coordinator asks the committee. Then the orange item "18 home games: schedule not entered" stays until the committee's schedule comes and is entered, which can take weeks. "Where we are going" asks for attention only when there is something to do. The idea's open question about the text ("ask the committee" or "enter the committee's schedule") was answered by keeping the text. That fits the work of entering, not the wait. Acceptable for this change if the coordinator accepts the wait. A way to say "asked, waiting" is not in scope and is not proposed here. Checked by a trace of story 2's behaviour against `vision.md`, "Where we are going".
- **S4. `change.md` no longer matches the spec.** Its summary still says "The coordinator sees how many club teams are known for each half", and the idea says the filter "Schedule not entered" on "Season" follows the same rule. The spec parks the list for each club team (`docs/ideas.md`, 2026-11-14) and leaves "Season" unchanged, passing it to the review of "Season". Each is a sound narrowing, but the summary must follow. Also, the review of "Season" (its idea) keeps the filter "Schedule not entered" and says it "follows the rule of" this change. After this change, the open item and the "Season" label "Not entered" show different games until that review decides. That is acceptable, because the label shows every home game, which is the larger set. Checked by reading `change.md`, `idea.md` and `docs/changes/2026-11-14-review-of-season/idea.md`.
- **S5. "The same teams as in the pull-down list" is loose.** In `docs/product/duties.md` the pull-down list also holds teams whose games left the game list and teams that a place still names. Story 1's first criterion ("a game ... that is still in the game list") narrows this correctly. Design should take the criterion as the rule, not the pull-down list. Checked by a search in `docs/product/duties.md` (pattern `pull-down`).

### Second spec review (2026-11-14, after the answer to S1)

Not in the arguments beyond `spec` and the folder: nothing ignored.

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| 1 – no reminder for a half season that is not ready | In December, looks at the start page and must not be asked for a schedule the committee cannot make yet | serves | The rule "a competition with its first game in that half" answers S1: MSE 1 (and VSE 1 in 2025-2026) never makes the second half ready. The example on 20 December now fails under the old rule (S2 solved). |
| 2 – all home games of a half season that is ready | Learns that the committee can start, asks for its schedule, then enters it game by game | serves – note | "· second half: 3 of 4 teams known" tells the coordinator whether to ask now or wait for one more team. Notes S6 (an example contradicts a criterion), S8 (the words of the new text). |
| Purpose line | – | serves | Core value 2 and "Where we are going" (attention only when there is something to do). Still supporting work and small: one rule and one text suffix, no new screen. |

**Overall: serves – note.** Design can start; S6 and S7 are edits to the documents, S8 is a
question for the coordinator that Design need not wait for.

- **S6. Story 2's last example contradicts its own criterion.** "3 club teams have a competition with its first game in the second half ... The item shows '18 home games: schedule not entered'." The criterion above it says that with home games of the second half the text adds "· second half: N of M teams known", as the Behaviour and the Interface table ("· second half: 3 of 4 teams known") show. The example must end with the suffix, or a test built from it fails one of the two. Checked by reading story 2 against its criteria and the Interface section.
- **S7. `change.md` states the old rule.** Its summary says "A club team counts for a half season when it has a game in it." That is the rule S1 rejected: with it MSE 1 counts for the second half. It also does not mention "N of M teams known". The summary is what `/design` and `/product-status` read first. Checked by reading `change.md` and `git diff` of it.
- **S8. "3 of 4 teams known": the words.** `docs/language.md` keeps "team" for the coordinator's team ("Say 'other club team' for the club's other teams"); the count is of club teams, the coordinator's team among them. The text also does not say which half is "known": in December, with home games of both halves in the one item, "18 home games: schedule not entered · second half: 3 of 4 teams known" does not say that the 3 of 4 is about the second half. A form such as "· second half: 3 of 4 club teams" would follow the word list and say it; the short form may be what the coordinator wants on a phone. A claim about the user's reading: not checked, ask the coordinator. Checked by a search in `docs/language.md` (pattern `club team`) and a trace of the December case in the Interface tables.
- **S9. Bruce's warning about the competition: partly settled.** The tool stores a competition for each game today: `Game.competition` in `crates/core/src/events/game.rs`, read from `cmp_naam` (the name) in `crates/server/src/source/parse.rs`, stored in `games.competition` (`crates/server/src/games.rs`). The source also has `cmp_id` and `cmp_nr`, which the tool does not keep. In the two test copies each `cmp_id` has its own name, both halves of a youth team have different names, and two club teams can share one (X14 1 and X14 2 in 2024-2025, per the evidence; not in these copies). Whether the name is safe as the key, or the number must be stored, is for Design. Checked by a search in `crates/` and `migrations/` (pattern `cmp_id|cmp_naam|competition`) and a script over `wedstrijden` in `games-37-2025-2026.json` and `games-37-2026-2027.json` (club teams, `cmp_id`, `cmp_naam`, first and last date).
- **S10. The data behind the examples holds.** In `games-37-2026-2027.json` (2026-10-11) MSE 1 has one competition from 12 September 2026 to 10 April 2027, every youth team one "1e Helft" competition ending by 12 December, and the first January home game is MSE 1 on Sat 9 January 2027; 20 December + 27 days is 16 January, so that game is in the old window. In `games-37-2025-2026.json` the four youth teams have a second-half competition from 17 January 2026 with another number and name, MSE 1 and VSE 1 one competition each: the rule gives 4 club teams for that second half and 6 for the first. The live list (X12 1 gone) was not checked. Checked by the same script and a date calculation.
- **S11. The first half widens today, a little.** From 14 November the old window ends on 11 December; the new rule adds the first-half home games of 12 December (the last day of the first half in the copy) if their schedule is not entered. The spec says so ("without the 27-day limit"). Whether these games already have "Schedule entered" in the coordinator's database: not checked (the database was not read). Checked by a list of home games in `games-37-2026-2027.json`.
