# Technical spec: Schedule not entered for each half season

**Status**: in progress
**Size**: story
**Change**: docs/changes/2026-11-14-schedule-per-half-season
**Date**: 2026-11-14

## Objective
The open item "schedule not entered" holds the coming home games of each half season that is
ready (more than 2 club teams have a competition that starts in it), with no 27-day limit, and
none of a half that is not ready; with home games of the second half its text adds how many club
teams are known for that half.

## Requirements summary
From `functional.md` (approved after two spec reviews):

- **Half season**: 1 July – 31 December (first), 1 January – 30 June (second), by the Dutch date
  of a game.
- **A club team counts for a half season** when one of its competitions has its first game in
  that half. The first game is the earliest game of that club team in that competition, home or
  away, still in the game list; a game marked "Not played" still counts. A game between two club
  teams counts for both; two club teams in one competition count as two.
  - A senior team with one competition for the season (first game in September) counts for the
    first half only. A youth team counts for the first half, and for the second half once its
    second-half competition (first game in January) is in the game list.
- **Ready**: more than 2 club teams count for the half.
- **The open item "schedule not entered"** (start page, item 4):
  - a half that is ready: every coming home game of the club of that half whose schedule is not
    entered and whose places count (`not_entered`), with no date limit;
  - a half that is not ready: none of its home games, also not within 27 days;
  - as now: no past games, no games out of the game list, no games that are no longer a home game
    of the club, no games marked "Not played"; the item and its list hold the same games; the text
    "Home game Sat 31 Oct: schedule not entered" / "18 home games: schedule not entered".
- **The count**: when the item holds a home game of the second half, the text adds
  "· second half: 3 of 4 teams known". Known = club teams that count for the second half; of =
  known + waiting, where waiting = club teams with games in the first half and none in the second
  half. A club team with games in both halves that does not count (a senior team) is in neither.
- **No other screen changes.** "Season" and its label "Not entered", the page of a game, the open
  item "open places" (27 days) stay as they are.

Examples that the tests follow (real game lists in `crates/server/tests/data/game-list/`):
- 2026-2027 on 20 December 2026: only MSE 1 (competition from 12 September) has games after
  1 January, so the second half is not ready; the home game of MSE 1 on 9 January 2027 is not in
  the item although it is within 27 days.
- 2025-2026 on 20 December 2025: M16 1, M18 1, X12 1 and X14 1 have a competition from 17 January
  2026; MSE 1 and VSE 1 play one competition for the season. So the second half is ready, with
  4 of 4 teams known.

## Current state
- `crates/core/src/output/start_page.rs:198` `schedule_not_entered(events, places, today) -> Vec<i64>`:
  the coming events (`SeasonEvents.coming`) from today until today + `DUTY_DAYS` (27) whose places
  are `not_entered`. `DUTY_DAYS` is also used by `open_places_games`. `OpenItem::ScheduleNotEntered
  { games: Vec<i64> }`; `open_items` takes the list as its third parameter.
- The coming events hold only the schedule (games of the team, home games of the club). The count
  of club teams needs all games of the club, also the away games of other club teams: those are the
  stored games of the season, `games::stored` (`Vec<StoredGame>`: `game`, `in_list`,
  `not_played_for`), read in `games_in` (`crates/server/src/api/season.rs:769`) and then moved into
  `schedule(...)`.
- `Game.competition: String` is the competition name from the source (`cmp_naam`,
  `source/parse.rs:155`); the competition number (`cmp_id`) is not stored. In the four seasons
  2023-2024 to 2026-2027 of club 37 each number has one name and each name one number.
- `Side { team, team_name, club }`: a side is a club team when `side.club` is the club of the
  choice.
- `SeasonGames` (`api/season.rs:718`): `state`, `club`, `team`, `schedule`, `unseen_changes`,
  `duties`. `GET /api/start` (`api/start.rs`) calls `schedule_not_entered(&events, &places, today)`
  at line 412 and maps `OpenItem::ScheduleNotEntered` to the view
  `ScheduleNotEntered { games: Vec<GameItemView> }` (line 136, 262).
- Front end: `web/src/lib/start.ts:44` makes the text of `schedule_not_entered`.
- Domains (decision 0027, `crates/core/tests/domains.rs`): `shared` uses nothing, `events` only
  `shared`, `output` all. `own_teams` is in `team`, which `events` may not use; `events` tests the
  club of a side itself.

## Steps

### Step 1: Core: the count for each half season
- **Goal**: the core knows the half of a date and, from the stored games of a season, the club
  teams known and waiting for each half, and whether a half is ready. Nothing uses it yet.
- **Changes**:
  - `crates/core/src/shared/season.rs`:
    ```rust
    /// The half of a season: 1 July – 31 December, or 1 January – 30 June.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Half { First, Second }
    impl Half {
        pub fn of(date: NaiveDate) -> Half // month >= FIRST_MONTH (7): First, else Second
    }
    ```
  - New `crates/core/src/events/half_season.rs`:
    ```rust
    /// For one half season: the club teams with a competition whose first game is in that half
    /// (`known`), and the club teams with games in the first half and none in the second
    /// (`waiting`; only for the second half, 0 for the first).
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct HalfCount { pub known: usize, pub waiting: usize }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct HalfSeasons { pub first: HalfCount, pub second: HalfCount }

    /// A half season is ready when more than this many club teams count for it.
    const READY_ABOVE: usize = 2;

    impl HalfSeasons {
        pub fn of(&self, half: Half) -> HalfCount
        pub fn ready(&self, half: Half) -> bool // self.of(half).known > READY_ABOVE
    }

    /// From the stored games of one season: only games with `in_list`, also "Not played". A
    /// side counts when `side.club == club`. Key of a competition: (team number, competition
    /// name), within the season.
    pub fn half_seasons(club: ClubNumber, games: &[StoredGame]) -> HalfSeasons
    ```
    Work: for each game in the list and each side of the club, keep the earliest date of
    (team, competition) and the halves in which the team has games. `known[h]` = teams with a
    competition whose earliest date is in `h`; `second.waiting` = teams with games in the first
    half and none in the second.
  - `crates/core/src/events/mod.rs`: `pub mod half_season;`. `crates/core/src/lib.rs`: the flat
    names `Half`, `HalfCount`, `HalfSeasons`, `half_seasons`.
- **Tests** (unit tests in the modules, made-up games):
  1. `Half::of`: 1 July and 31 December first; 1 January and 30 June second.
  2. A senior team with one competition September – April: first known 1; second known 0,
     waiting 0.
  3. A youth team with a first-half competition only: first known 1, second waiting 1. With a
     second competition from January: second known 1, waiting 0.
  4. A team with only away games counts.
  5. A game between two club teams counts for both; two club teams in one competition count as
     two.
  6. A "Not played" game counts; a game with `in_list = false` does not; when the first game of a
     competition is out of the list, the next game decides the half.
  7. A team with games only in the second half: second known 1, not waiting.
  8. `ready`: known 2 is not ready, known 3 is ready.

### Step 2: Core and server: the rule of the open item
- **Goal**: the open item holds the home games of the ready halves, with no date limit, and the
  count of the second half; the API sends it.
- **Changes**:
  - `crates/core/src/output/start_page.rs`:
    ```rust
    pub struct NotEntered { pub games: Vec<i64>, pub second_half: Option<HalfCount> }
    pub fn schedule_not_entered(
        events: &SeasonEvents<'_>,
        places: &PlacesOf,
        halves: &HalfSeasons,
    ) -> NotEntered
    OpenItem::ScheduleNotEntered { games: Vec<i64>, second_half: Option<HalfCount> }
    ```
    The coming game events whose places are `not_entered` and whose `Half::of(event.date)` is
    `ready`, in the order of `events.coming` (time order, as now). `second_half` is
    `Some(halves.second)` when one of the games is in the second half, else `None`. The 27-day
    limit goes from this function; `DUTY_DAYS` stays for `open_places_games`. `open_items` takes
    `NotEntered` in place of `Vec<i64>` and pushes the item when its games are not empty. Update
    the doc comments (the item is no longer "4 weeks ahead").
  - `crates/server/src/api/season.rs`: `SeasonGames` gets `pub halves: HalfSeasons`; `games_in`
    sets it with `half_seasons(club.number, &stored)` before `schedule(stored, ...)`; without a
    team `HalfSeasons::default()` (no half ready).
  - `crates/server/src/api/start.rs`: `schedule_not_entered(&events, &places, &games.halves)`;
    the view
    ```rust
    ScheduleNotEntered { games: Vec<GameItemView>, second_half: Option<SecondHalfView> }
    /// The club teams known for the second half, of the known and the waiting club teams.
    pub struct SecondHalfView { pub known: usize, pub of: usize } // of = known + waiting
    ```
    (with the `utoipa` schema derive as the other views). Update the doc comment of the view.
  - `web/openapi.json` and `web/src/lib/api/schema.d.ts` made again (`just api-types-check` checks
    them).
  - Other callers of `games_in` / `SeasonGames` build: check with
    `cargo clippy --workspace --all-targets -- -D warnings`.
- **Tests**:
  9. Core: a half that is not ready (known 2): its home game 10 days ahead is not in the item.
  10. Core: rewrite `schedule_not_entered_covers_the_home_games_of_the_next_4_weeks` as "covers
      the coming home games of a ready half": with the first half ready, game 4 (23 Nov, past the
      old limit) is in it; gone, entered and swapped games stay out.
  11. Core: `second_half` is `None` with first-half games only, `Some` when a second-half game is
      in the item; with both halves ready the item holds the home games of both.
  - Adapt `a_not_played_game_is_no_open_item_but_stays_in_next` (`start_page.rs:1196`) and
    `open_items_come_in_the_order_of_the_spec` to the new parameter.
  12. Server, `crates/server/tests/start.rs`: game list 2026-2027 (`games-37-2026-2027.json`),
      clock on 20 December 2026: the home game of MSE 1 on 9 January 2027 is not in the item;
      `second_half` is null. Fails with the old rule.
  13. Server: game list 2025-2026 (`games-37-2025-2026.json`), clock on 20 December 2025: the
      item holds home games of the second half and `second_half` is `{ known: 4, of: 4 }`.
  - Write 12 and 13 first: check that the helpers (`Reader`, `at`) can choose the season of
    2025-2026 with its file; if not, add a small helper, do not leave the test out.

### Step 3: Front end: the text
- **Goal**: the coordinator sees the count of the second half.
- **Changes**: `web/src/lib/start.ts`, case `schedule_not_entered`: add
  `` ` · second half: ${known} of ${of} teams known` `` when `item.second_half` is present, for one
  game and for more.
- **Tests**:
  14. `web/e2e/duties.e2e.ts` (the test with the replaced answer of `/api/start`): with
      `second_half: { known: 3, of: 4 }` the item shows "18 home games: schedule not entered ·
      second half: 3 of 4 teams known", for one game "Home game Sat 9 Jan: schedule not entered ·
      second half: 3 of 4 teams known"; `expectAccessible` as now. The cases without
      `second_half` stay.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| A club team counts for a half season when one of its competitions has its first game in it; ready above 2 club teams; 1 January splits the season | The coordinator's rule; senior teams with one competition never make the second half ready; seen in four seasons | Yes, 0037 at Close |
| The competition name is the key of a competition, within one season; no competition number, no migration | Each name has one number and each number one name in four seasons; the rule uses only the games of the current reading, which writes all of them again | Yes, in 0037 |
| The rule lives in `events` (count) and `shared` (half), the item in `output` | `events` needs only games and `shared`; `output` may use all; no change of the domain map | No |
| The count is worked out at each load, nothing stored | No stale state after a move or a game that leaves the list (Discover warning) | No |
| The open item "open places" keeps its 27 days | Not in the spec | In 0037 |

## Not in scope
- The competition number (`cmp_id`) and a migration.
- Any change to "Season", its label "Not entered" or the rows "Schedule not entered".
- The list for each club team (parked in `docs/ideas.md`).
- The open item "open places".
- A setting for the threshold.

## Test plan
- **Existing tests that must still pass**: all of `cargo test -p team_core` and
  `cargo test -p team --test start` (`a_held_back_reading_is_an_open_item` keeps
  `schedule_not_entered`: the first half of 2026-2027 has 6 club teams); `web/e2e/duties.e2e.ts`
  and `web/e2e/choose.e2e.ts` (its `\d+ home games` now counts all home games of the first half
  from 12 October, not only 27 days); the domain test `crates/core/tests/domains.rs`.
- **New tests**: 1–8 (step 1), 9–13 (step 2), 14 (step 3).
- While working: `cargo test -p team_core`, `cargo test -p team --test start`; then
  `cargo clippy --workspace --all-targets -- -D warnings`, `just api-types-check`,
  `just check-web`, `cd web && npx playwright test duties choose`. `just check` once before the
  commit.

## Compatibility and performance
- After the release the first half (6 club teams) is ready at once: its home games until
  31 December without a schedule come into the item, not only the next 27 days (spec review S11).
  The home games of the second half (only MSE 1) leave it.
- The API answer of `GET /api/start` gets the field `second_half` (null or an object); the front
  end is released with it.
- No new queries: the stored games are already read; one pass over them (about 70–120 games a
  season).

## Progress
- [x] Step 1 (2026-11-14: `Half::of`, `events/half_season.rs`; 11 unit tests)
- [x] Step 2 (2026-11-14: `schedule_not_entered` with `HalfSeasons` and `NotEntered`, `SeasonGames.halves`, `SecondHalfView`; 2 core tests added and 1 rewritten, 2 server tests on the real game lists; test 12 fails when every half is ready)
- [ ] Step 3
- [ ] All checks pass

## Deviations
- 2026-11-14 (Implement, step 2): the test plan missed one existing test of the 27-day limit,
  `schedule_not_entered_lists_the_home_games_of_4_weeks_and_shrinks` in
  `crates/server/tests/games.rs`. Renamed to `..._of_the_ready_half_and_shrinks`; it now expects
  the coming home games of the first half until 31 December, with the home games of 14 November,
  and no count of the second half. No change of behaviour.

The design reads "the club teams" of story 1 as the sides of the games still in the game
list whose club is the club of the choice; that is the criterion of the spec (spec review S5).

## Review findings
