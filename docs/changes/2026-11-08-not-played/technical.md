# Technical spec: Not played

**Status**: approved
**Size**: story
**Change**: [2026-11-08-not-played](./)
**Date**: 2026-11-08

## Objective
The coordinator marks any game "Not played" with one tap, so that its duties and attendance stop
counting and it gives no warning, open item or message line, while everything stored stays and
can still be changed.

## Requirements summary
From story E in `../2026-10-31-attendance-at-games/functional.md` (approved; changed with the
coordinator on 2026-11-08), with the answers of this design:

- **The button.** Every game whose page opens has "Not played" (D3): a game of the team, a home
  game of another club team, a kept game, coming or past, also a game that left the game list.
  It sits below "Checked" when that is there (story C), else as the last button.
- **One tap** saves at once. Then:
  - the table and referee places and the attendance stop counting;
  - the driver places stop counting, except from the start time with "The drivers drove" on;
  - no warning (too few players, driver not coming, moved game) and no open item (the three
    warnings, schedule not entered, open places);
  - the game leaves the message of the week (its block too), the committee message and both duty
    overviews;
  - a not played game blocks no player in the clash rule (as a game of the team or by its duty
    places).
- **The page** shows "Not played" with "Undo" at the top; the list of players is closed behind a
  tap. Answers, extra players and duty places can still be changed. "Undo" saves at once and
  makes everything count as before.
- **The switch "The drivers drove"**: shown from the start time (without a start time: from the
  start of the game day) on a not played game with driver
  places of the team, off at first. On: the driver places count as at a played game (done by the
  player, with "Someone else" and "Nobody"). Off: no drive counts. "Undo" keeps it stored; it has
  effect again only after a new "Not played".
- **Rows**: on "Season", in "Next" and in "Last" the row shows "· Not played"; on "Season" and in
  "Next" a game of the team shows it instead of the count. A not played game stays in "Next" and "Last".
- **A mark holds for a date** (decision 0029): when the game gets another date it is a normal
  game again, with its answers and duties. A reading never sets or clears the mark.
- **Errors**: a save that fails: "Not saved. Try again." and the stored state. Two screens: the
  last tap wins.

Rules of the coordinator (2026-11-08) that this design follows: everything entered can always be
changed; "in the past" changes only how events are shown; "not played" changes only what counts;
"Checked" means the admin is complete and locks nothing. Each domain owns its tables and is
treated as if it were a service: only its store module reads or writes them; foreign keys between
domains and one transaction through the functions of two domains are allowed.

## Current state
- Every page and message gets the games of a season from `games::stored`
  (`crates/server/src/games.rs:14`) as `team_core::StoredGame { game, in_list }`
  (`crates/core/src/events/schedule.rs`); `schedule()` turns them into `ScheduledGame`.
- `game_places` (`crates/core/src/contribution/duties.rs:272`) sets
  `counts = past || (in_list && required)` for the schedule part and the driver part. Readers of
  `counts`: `place_counts` (the duty totals and the duty part of `clashes`), `open_by_kind`,
  `not_entered`, `has_team_places`, and in `messages.rs` `drivers_of_team` and `places_of_team`
  (drivers and table/referee lines of all four messages).
- `has_team_places` is used by `last_games` and `next_events` (`crates/core/src/output/start_page.rs:236`)
  to show games of other club teams with places of the team.
- `week_message` selects the game blocks by `TeamGame && in_list && date >= today`, not by
  `counts` (`messages.rs:259`).
- `clashes` takes `team_games` (games of the team in the game list), made in the server in
  `SeasonPlaces::new` (`crates/server/src/api/season.rs:421`).
- `game_warnings` (`crates/core/src/contribution/game_attendance.rs:220`) gives nothing for a game
  that is not in the game list; the page and the open items use it (decision 0022).
- `StoredDuties` comes from `game_duties` (`duties::for_season`); `duties::lock` takes the row
  lock of a game for each duty write.
- Tables by store module: `games`/`places` only in `games.rs`, duties tables only in
  `duties.rs`, attendance tables only in `game_attendance.rs`. Migrations end at `0013`.

## Steps

### Step 1: The mark is stored (events)
- **Goal**: the mark can be set and undone and is read with the games; nothing counts
  differently yet.
- **Changes**:
  - Migration `0014_games_not_played.sql`:
    ```sql
    -- A game that was not played on a date (story E of attendance at games). Events owns it;
    -- a reading of the game list never writes it.
    CREATE TABLE games_not_played (
        club_number    bigint  NOT NULL,
        season_year    integer NOT NULL,
        source_id      bigint  NOT NULL,
        -- The date of the game when "Not played" was tapped (decision 0029).
        not_played_for date    NOT NULL,
        PRIMARY KEY (club_number, season_year, source_id),
        FOREIGN KEY (club_number, season_year, source_id) REFERENCES games
    );
    -- The switch "The drivers drove" (duties owns it; used from step 2).
    ALTER TABLE game_duties ADD COLUMN drivers_drove boolean NOT NULL DEFAULT false;
    ```
  - Core, `events/schedule.rs`:
    ```rust
    pub struct StoredGame { pub game: Game, pub in_list: bool, pub not_played_for: Option<NaiveDate> }
    pub struct ScheduledGame { pub game: Game, pub reason: ScheduleReason, pub in_list: bool, pub not_played: bool }
    /// The mark holds for the date of the game now (decision 0029).
    pub fn not_played(game: &Game, not_played_for: Option<NaiveDate>) -> bool {
        not_played_for == Some(game.date)
    }
    ```
    `schedule()` fills `not_played`. Re-export in `lib.rs`.
  - `games.rs`: `stored()` reads `not_played_for` with a `LEFT JOIN games_not_played`.
    `set_not_played(db, key)`: one statement, `INSERT … SELECT date FROM games WHERE <key>
    ON CONFLICT DO UPDATE SET not_played_for = EXCLUDED.not_played_for`. `unset_not_played(db,
    key)`: `DELETE`. `write()` does not touch the table.
  - Routes in `api/games.rs`, registered in `app.rs` and `api/mod.rs`:
    `PUT /api/games/{season}/{id}/not-played` and `DELETE …/not-played`. Each answers the page
    (`GamePage`), as the other writes. A game is found by `find` (404 when the page does not
    open); any found game can be marked (D3).
  - Views: `GamePage.not_played: bool`; `ScheduleGame.not_played: bool` (`api/season.rs`).
  - `just prepare`, `just api-types`.
- **Tests**:
  - Core: `not_played` with the date of the game, another date, no mark.
  - `crates/server/tests/games.rs`: "Not played" stores the date of the game now; "Undo" removes
    it; a reading (also one that removes the game from the game list or moves it) leaves the row
    as it was; a moved game has `not_played: false` with the row still stored.
  - Routes: 401 without login; 404 for an unknown game; a home game of another club team without
    places of the team can be marked; "Not played", "Undo", "Not played" gives `not_played: true`.
  - Logs: the new routes log no player name (pattern of `warning_logs.rs`).
  - Check that the browser test data has a past away game of the team with driver places; if not,
    add one (needed in step 4).

### Step 2: Duties stop counting (contribution)
- **Goal**: the duty totals, open places, clashes and duty lines leave out a not played game,
  except the drives with the switch on.
- **Changes**:
  - `StoredDuties.drivers_drove: bool`, read in `duties::for_season`.
  - `game_places(…, past, not_played: bool, started: bool)`; `started`: the start time has passed
    (without a start time: the game day has begun), from `!before_start(game, now)` in Dutch time. Each part gets `in_use` beside `counts`:
    ```rust
    let in_use = |required: bool| past || (in_list && required);   // as played
    schedule.counts = !not_played && in_use(required.schedule);
    drivers.counts  = if not_played { started && stored.drivers_drove } else { in_use(required.drivers) };
    ```
    `has_team_places` reads `in_use` (so a not played game stays in "Next" and "Last");
    `open_by_kind`, `not_entered`, `place_counts` and the messages keep reading `counts`.
    All callers pass `scheduled.not_played` / `stored.not_played()` (`api/season.rs`
    `places_of`, `api/games.rs` `Found::places`, test helpers).
  - `pub fn team_games<'a>(stored: &'a [StoredGame], team: TeamNumber) -> Vec<&'a Game>` in
    `contribution/duties.rs`: in the game list, the team plays, not not played.
    `SeasonPlaces::new` uses it.
  - `duties::set_drivers_drove(db, key, on)` under `duties::lock` (the row of `game_duties` is
    made as in `set_drivers`). Routes `PUT` and `DELETE /api/games/{season}/{id}/drivers-drove`,
    answering the page; accepted on any found game (it has effect only on a not played game from its start time
    with driver places). `GamePage.drivers_drove: bool`.
- **Tests**:
  - First: `has_team_places` is true for a not played game with places of the team.
  - `game_places`: coming not played (no part counts, `in_use` unchanged); started not played
    (table/referee never count; drivers only with the switch); not played before the start time
    with the switch on (drivers do not count).
  - `duty_totals`: a not played past game gives no "done"; with the switch, only the drives;
    without the mark the totals are as before.
  - `clashes`: a not played game of the team blocks nothing; its duty places block nothing.
  - The existing test `a_coming_game_that_is_gone_does_not_count_and_a_past_one_does` passes.
  - `crates/server/tests/duties.rs`: the switch is stored; it stays after "Undo"; it has effect
    again after a new "Not played"; the totals of the page "Duties" before the mark and after
    "Undo" are the same.

### Step 3: Warnings, open items and messages
- **Goal**: a not played game warns about nothing and is in no message.
- **Changes**:
  - `game_warnings(…, not_played: bool)`: `GameWarnings::default()` when not played. Callers in
    `api/start.rs` and `api/games.rs`; on the page also `answers_moved: false` for a not played
    game.
  - `week_message`: the block filter also needs `!game.scheduled.not_played`.
  - The committee message and both overviews follow `counts` (step 2); only tests.
  - The start page open items `schedule_not_entered` and `open_places_games` follow `counts`;
    only tests.
- **Tests**:
  - Core: `game_warnings` for a not played game in the warning period with 5 coming and a driver
    "not coming": nothing. `week_message`, `committee_message`, `driving_overview`,
    `table_referee_overview` with a not played game of this week: no block, no drivers, no lines;
    as the only game: no message of the week, and the committee message says
    "Geen tafel- of scheidsrechtersdiensten deze week.". `open_places_games` and
    `schedule_not_entered`: no item. `last_games` and `next_events`: the game stays.
  - `crates/server/tests/start.rs`: no open item for a not played game with too few players and a
    driver "not coming"; `not_played: true` in "Next" and "Last".
  - `crates/server/tests/season.rs`: the row has `not_played: true` and no "Open places".
  - `crates/server/tests/messages.rs`: a not played game of this week is not in the message.

### Step 4: The page and the rows (front end)
- **Goal**: the coordinator taps "Not played", "Undo" and the switch, and sees "· Not played" on
  the rows.
- **Changes**:
  - `web/src/lib/api/client.ts`: `setNotPlayed`, `unsetNotPlayed`, `setDriversDrove`,
    `unsetDriversDrove`.
  - The page of a game (`web/src/routes/games/…/+page.svelte`): at the top "Not played" with
    "Undo"; the list (`GameAttendance.svelte`) closed behind a tap while not played; the button
    "Not played" as the last button (below "Checked" once story C adds it); from the start time, on a
    not played game with driver places, the switch "The drivers drove". Each tap saves at once,
    with "Not saved. Try again." and the stored state on failure.
  - Rows on "Season", "Next" and "Last": "· Not played"; on "Season" and in "Next" instead of the
    count.
- **Tests** (`web/e2e/game-attendance.e2e.ts`, `start.e2e.ts`, `season.e2e.ts`, at 320 pixels):
  - "Not played" shows "Not played" with "Undo"; the list is closed; a tap opens it and an answer
    can still be changed; "Undo" brings the page back.
  - From the start time, an away game shows the switch and it saves; before the start time it is
    not shown.
  - A failed save (`page.route`): "Not saved. Try again." and the stored state.
  - "· Not played" on "Season" and in "Next", instead of the count.
  - No horizontal scroll at 320 pixels.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| Each domain owns its tables, treated as if it were a service; foreign keys between domains and one transaction through the functions of two domains are allowed | Clear boundaries without separate services (coordinator, 2026-11-08) | Yes, 0030 at close; known debt: `trainings.rs` reads `player_roles`, `trainings` holds two domains |
| "Not played" is a fact of the game, in events, in its own table | A game exists without attendance and duties; a reading must never change it, so not on `games` | Yes, 0031 at close (planned in the epic) |
| The mark on `StoredGame`, not a separate list | Each reader gets it from the type it already uses; a forgotten input cannot count the game | Same record |
| `counts` and `in_use` on each part of `GamePlaces` | What counts and whether the game shows with places of the team are different questions | Same record |
| The switch in `game_duties` | It is a fact about duties | Same record |
| Few refusals: any found game can be marked, the switch is accepted at any time | Everything entered can always be changed (coordinator) | No |

## Not in scope
- "Checked" for games and its rules with "not played": story C. The epic log notes two criteria
  of story C to ask again.
- The games in the totals of "Attendance": story D.
- "Someone else" before the game (`docs/ideas.md`, 2026-11-08).
- One action for all games of a club team that stopped playing (`docs/ideas.md`, 2026-11-01): one
  tap per game for now.
- Moving the read of `player_roles` out of `trainings.rs`, and a test that checks the SQL of each
  domain: small lines in `docs/backlog.md`.

## Test plan
- **Existing tests that must still pass**: all core tests (`cargo test -p team_core`), notably
  `a_coming_game_that_is_gone_does_not_count_and_a_past_one_does` and the tests of
  `has_team_places`; `crates/server/tests/`; `web/e2e/`.
- **New tests**: per step, see above.
- While working: the tests of the changed part; `just check` once before the commit.

## Compatibility and performance
- Forward-only migration with a new table and a column with a default; no data to move. Nothing
  changes for a game that is not marked.
- Released rules change only for marked games: the duty totals, the messages, the overviews, the
  clashes and the open items.
- One `LEFT JOIN` more in `games::stored`; the seasons have a few hundred games.

## Progress
- [ ] Step 1: the mark is stored
- [ ] Step 2: duties stop counting
- [ ] Step 3: warnings, open items and messages
- [ ] Step 4: the page and the rows
- [ ] All checks pass

## Deviations
- **D1** (from the epic `technical.md`): the mark is in its own table `games_not_played` of events,
  and the switch is in `game_duties`, not both in `game_attendance`. Reason: each domain owns its
  tables (coordinator, 2026-11-08).
- **D2** (from `functional.md` as first approved): the switch "The drivers drove" and "can still be
  changed" are new criteria of story E, added to `functional.md` with the coordinator on
  2026-11-08.
- **D3** (from `functional.md`): the button is on every game whose page opens, not only on games of
  the team and other games with places of the team. Reason: "not played" changes only what counts,
  and a club team that stopped playing leaves home games that ask for places
  (`docs/ideas.md`, 2026-11-01) (coordinator, 2026-11-08).
- Stricter readings: none.

### Spec review of the deviations
Fresh review, 2026-11-08, mode `spec` with `deviations`. Arguments held nothing else to ignore.
Read: `~/.claude/skills/product-workflow/SKILL.md`, `docs/language.md`, `docs/vision.md`, story E
and the reviews in `../2026-10-31-attendance-at-games/functional.md` (with its uncommitted diff),
`docs/product/duties.md`, and the lines of `start-page.md`, `season.md`, `schedule.md` and
`game-attendance.md` found by the search named below; this file only for its requirements and
deviations. Not read: `docs/changes/done/`. The question for each deviation: what is the
coordinator doing here, and does the difference serve that, and the purpose?

| Deviation | What the coordinator is doing here | Verdict | Note |
|---|---|---|---|
| D1: the mark in `games_not_played` (events), the switch in `game_duties` | Nothing: the coordinator never sees where a fact is stored | serves | No difference for the user. Both facts survive a reading and "Undo", which is what story E asks ("a reading never sets or clears the mark", "Undo keeps the switch stored"). Checked: search `DELETE FROM games\|ON DELETE` in `crates/server/src` and `migrations`: no code deletes a game row, so the foreign key to `games` cannot block a reading or lose a mark. |
| D2: the switch "The drivers drove", and "can still be changed" | After an away game that was called off at the hall: making sure the parents who drove get the drive, while nobody gets a table, referee or absence for it; and later fixing an answer or a place on that game | serves – note | Serves core value 2 (a fair share of the duties): without the switch, "Not played" would take the drive away from parents who did drive, and the only fix would be not to mark the game, which brings the absences back. Off at first is the right side to err on: a forgotten switch costs a drive that the coordinator can add, not a duty nobody did. "Can still be changed" matches the coordinator's rule and the word list ("not played" stops counting, nothing is deleted). Note N1. |
| D3: the button on every game whose page opens | Entering the committee schedule and finding home games of a club team that stopped playing, which ask for 4 places and give "Schedule not entered" each week | serves – note | Fits the case in `docs/ideas.md` (2026-11-01) exactly, and the word list ("a game that did not, or will not, take place") is not tied to the team. Fewer refusals is cheaper than a rule about which games may be marked. Note N2. |

No deviation conflicts with the vision: nothing is sent, a reading changes nothing stored, and
each deviation makes the totals closer to what happened.

**Notes**
- **N1 (D2). The switch is not there on the day the game is called off.** The case behind the
  switch is a game that falls through when the team is already at the hall, so the coordinator
  learns it on the day itself, and will likely tap "Not played" that evening. By "after its date"
  (and `past` = before today, as "After the game" in `duties.md`) the switch shows only from the
  next day, so the coordinator must come back to the game. Nothing reminds them: a not played game
  is no open item, and "Last" shows it without the switch. The drive is then lost silently, which
  is the error D2 exists to prevent. The rule follows the existing "after the game" moment of
  "Someone else" and "Nobody", so it is consistent, and the server already accepts the switch at any
  time; only the page hides it. Options for the coordinator: show the switch from the start time of
  the game (as "Checked" in story C), or keep "after the date" and accept the return. Checked:
  trace of story E against `duties.md` ("After the game", "Done and planned"); how often a game is
  called off at the hall is not checked.
  - 2026-11-08: Fixed: the switch shows, and the drives count, from the start time (without a start time: from the start of the game day); `game_places` gets `started`; functional.md changed (coordinator).
- **N2 (D3). One tap per game, and a mark with no visible owner.** A club team that stopped playing
  may leave 8 to 10 home games: one tap each, as the "Not in scope" line says, with the idea of one
  action for a club team still open in `docs/ideas.md`. Acceptable for a first release. A mark on a
  game of another club team without places of the team shows only on "Season" ("· Not played"),
  since "Next" and "Last" show only games with places of the team; that is enough to find and undo
  a wrong tap. Checked: `start-page.md` ("Last", "Next"), `season.md` ("Which events"); the number
  of such games is not counted.

  - 2026-11-08: Kept: one tap per game for now (coordinator).

**Seen outside the deviations (not judged).** The requirements summary says that in "Next" a coming
game of the team shows "· Not played" instead of the count, but says nothing of the count on the
row of the same game on "Season", which story A shows once the game has an answer. Story E names
only "· Not played" for both. Worth one line in step 4, so the two rows agree. Checked: read of the
requirements summary against story A ("The row of a coming game …") and story E.

## Review findings
  - 2026-11-08: Fixed: on "Season" too a not played game of the team shows "· Not played" instead of the count (coordinator).
