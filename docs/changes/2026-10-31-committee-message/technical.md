# Technical spec: The committee message

**Status**: done
**Size**: story
**Change**: [2026-10-31-committee-message](./)
**Date**: 2026-10-31

## Objective
The coordinator copies one Dutch message with the team's table and referee places of this week and
the full names of the players, so the referee committee knows who does which duty.

## Requirements summary
Story D of the epic [Duties](../2026-10-18-duties/) (`functional.md`, change of 2026-10-30, changed
2026-10-31); the week and the past places follow decision 0025.

- **The start page**: the part "Messages" has "Committee message" whenever a team is chosen for the current season, also in a week without places. So the part "Messages" always shows with a team.
- **The page** (`/messages/committee`): the text, "Copy", "Copied", "Select the text and copy it.", "Back" to the start page. No week buttons.
- **The text**: the heading with the team and the week, then each game of this week with table or referee places of the team that count, in time order, with one empty line between games. Full names; open places as "nog N open"; past places show who did it, or "niemand". Places of other club teams are never mentioned.
- **A week without places**: the heading and "Geen tafel- of scheidsrechtersdiensten deze week."; "Copy" still works.

**Readings of the spec, confirmed by the coordinator (2026-10-31):**
- **R1.** A week with only a past place that stayed open shows that game with "niemand": "Geen …" would be false, because the team had a place.
- **R2.** The heading has the club name and the short team name: "Blue Drakes M16 1", as the start page shows the chosen team.

**Readings made exact here:**
- **R3.** Without a chosen team: `{ "text": null }`, no button; the page says "No message: no team chosen."
- **R4.** The heading has both full dates, as the message of the week.
- **R5.** Two players with the same full name are not told apart.

## Current state
- `crates/core/src/messages.rs`: `week_of`, `day_and_month`, `duty_lines(game, names, today)`, `week_message`. `display_names` is in `duties.rs`.
- `crates/server/src/api/messages.rs`: `MessageView`, `week_text`, `week`. `SeasonGames` has `club: Option<Club>`, `team: Option<Team>` (short name) and `duties.players` (all players of the season, with their names).
- `api/start.rs`: `MessagesView { week }`. Front end: `web/src/routes/messages/week/`, `client.ts`, the part "Messages" in `+page.svelte`, which shows only with `messages.week`.
- Tests: core `messages`, `tests/messages.rs`, `tests/message_logs.rs`, route lists in `tests/players.rs` and `tests/player_logs.rs`, `web/e2e/messages.e2e.ts`.

## Steps

### Step 1: The text in the core
- **Goal**: the one rule of the committee message, every form fixed by an exact-text test.
- **Changes** in `crates/core/src/messages.rs`, exported from `lib.rs`:
  ```rust
  /// First and last name of `players`; a player without a last name: the first name.
  pub fn full_names(players: &[Player]) -> BTreeMap<PlayerId, String>;

  /// The committee message of the week that holds `today`. `games`: as `week_message`; `names`:
  /// `full_names` of the season; `team_name`: "Blue Drakes M16 1".
  pub fn committee_message(
      games: &[MessageGame<'_>],
      names: &BTreeMap<PlayerId, String>,
      team_name: &str,
      today: NaiveDate,
  ) -> String;
  ```
  It uses a private `week_games(games, today)`, shared with `week_message` (the filter on the week and the stable sort), and `duty_lines`. It always gives a text.
- **The text forms**:
  ```
  Tafel en scheidsrechters Blue Drakes M16 1, week 26 oktober – 1 november

  za 31 okt 13:00 Blue Drakes X12 1 – De Dunckers X12 1
    Scheidsrechter: Eva de Vries
    Tafel: Fleur Jansen, Gijs Bakker

  zo 1 nov 11:00 Blue Drakes M14 2 – Lions M14 1
    Scheidsrechter: nog 1 open
    Tafel: Hugo Smit
  ```
  Without places:
  ```
  Tafel en scheidsrechters Blue Drakes M16 1, week 26 oktober – 1 november

  Geen tafel- of scheidsrechtersdiensten deze week.
  ```
- **Tests** (core, exact text):
  1. The example above, character for character (two games, given in the wrong order).
  2. No places: the second form; also with only places of other club teams.
  3. `full_names`: first and last name; no last name; a last name with a prefix ("Eva de Vries").
  4. Open places: "Eva de Vries, nog 1 open", "nog 2 open".
  5. A past place: "Someone else", "Nobody" as "niemand", a place that stayed open as "niemand", also alone (R1).
  6. A coming game no longer in the game list or no longer a home game of the club is left out; a past one that left the game list stays.
  7. A game of the team without places is not in it.
  8. Sunday: the whole week; the next Monday is not in it.

### Step 2: The route and the start page flag
- **Goal**: the server gives the text and tells the start page, from one function.
- **Changes**:
  - `api/messages.rs`: a private `message_games(games, places, today)` with the mapping that `week_text` has now; `committee_text(games, places, today) -> Option<String>`, `None` without a club or a team, with `full_names(&games.duties.players)` and `format!("{} {}", club.name, team.name)`; `GET /api/messages/committee` (operation `committeeMessage`), `MessageView`.
  - `api/start.rs`: `MessagesView { week, committee }`, with `committee` from `committee_text(...).is_some()`.
  - `app.rs`, `api/mod.rs`, `just api-types`.
  - Read only; the text holds full names and is never logged.
- **Tests** (`tests/messages.rs`, `tests/message_logs.rs`, route lists):
  1. Without a login: 401. Without a team: `null`, `messages.committee` false.
  2. Exact text from stored data, with first and last names.
  3. A week without places: the "Geen …" text, `messages.committee` true.
  4. The log has no first or last name of the message.

### Step 3: The shared page and the button
- **Goal**: the coordinator reaches the message; the pages of the messages share one component.
- **Changes**:
  - New `web/src/lib/components/MessagePage.svelte` with the props `title`, `back`, `empty` and `message`: the markup and the "Copy" logic of `web/src/routes/messages/week/+page.svelte`.
  - `/messages/week` uses it. New `/messages/committee` (`+page.svelte`, `+page.ts`), empty text "No message: no team chosen."
  - `client.ts`: `loadCommitteeMessage`. `+page.svelte` of the start page: the part "Messages" shows with `week` or `committee`; the button "Committee message".
- **Tests** (`web/e2e/messages.e2e.ts`), on the real routes, because this message exists in every week:
  1. The start page opens the committee message; the text is the "Geen …" text of the test week; "Back" returns.
  2. "Copy" puts the text on the clipboard.
  3. Phone width and accessibility.
  4. The test "without a message" changes: without a message of the week the start page shows "Committee message" and not "Message of the week".

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| A second function in the core on `duty_lines` | Decisions 0024, 0025 | No |
| One page component for all texts to copy | Four pages with the same behaviour after story E | No |

## Not in scope
- The duty overviews (story E). Another week than this one.

## Test plan
- **Existing tests that must still pass**: all; the test "without a message" changes in step 3.
- **New tests**: per step, above.

## Compatibility and performance
- Nothing changes for existing data. `GET /api/start` gets one field and does no new query.
- Full names leave the encrypted store only in this text, as a player's name does on other pages (decision 0021); never in a log.

## Progress
- [x] Step 1
- [x] Step 2
- [x] Step 3
- [x] All checks pass

## Deviations
None from `functional.md`. Steps 1 to 3 go into one commit: step 2 changes the generated front-end types.

## Review findings
Build review, 2026-10-31, by a fresh agent. The arguments held only the mode and the change folder;
nothing was ignored. Read: the diff against `main` (committed and not), the change folder, story D
of the epic's `functional.md`, `docs/architecture.md`, decisions 0007, 0021 and 0023 to 0025,
`CLAUDE.md`. **No bug against the spec's intent, no broken decision. R1 to R5 are small; none blocks
Close.**

**Checked and sound:**
- **Runs:** `cargo test --workspace` 597 passed, `cargo fmt --check`, clippy `-D warnings`, `just check-web`, `just e2e` 125 passed. `web/openapi.json` is equal to a fresh `team openapi` output (written to a temporary file, compared as JSON); `schema.d.ts` was not regenerated, because `just api-types-check` writes to the repository. Not run: `lint-ci` (no workflow or script in the diff), `check-sqlx` (no query in the diff), `just images`, `just smoke`. So "All checks pass" holds for everything the diff can touch.
- **Trace, names:** `SeasonDuties.players` holds every player with a role in the season or on a place, also "Someone else" and hidden players (`load_duties`), and `names` is made from the same list, so the "?" of `filled_and_open` cannot show in this message.
- **Trace, order:** `schedule.past` is newest date first, in time order within a date; the stable sort on the date in `week_games` keeps that, so games come in time order on any weekday.
- **Trace, what counts:** a game today is coming and shows "nog N open"; `check_outcome` refuses an outcome before a game is past, so no correction of a game today is lost.
- **Trace, access and logs:** the route is under the session of decision 0007, and is in both route lists; the text is not logged (`tests/message_logs.rs`, first and last name). `GET /api/start` does no new query, as claimed: `committee_text` uses what `games_of` already read.
- **Search** for `messages/week` in `crates`, `web`, `docs/product`, `docs/architecture.md`, `.claude`, `CLAUDE.md`, `README.md`: no other route list missed. Search for `'messages'` in `web/e2e`: no other test relies on the part being absent.
- **The tests can fail:** read against the code; each step 1 and step 2 test pins something the code could get wrong (the week filter, `counts`, `of_team`, full against shown names, the flag). No mutation was run.

**R1 – Comments the diff made false.** (a) `crates/server/src/api/start.rs:44`: "The messages that have something to say." The committee flag is true also in a week with nothing to say; the line goes into `web/openapi.json` too. (b) `crates/core/src/messages.rs:1` still says "(story C of duties)"; its twin in `api/messages.rs` was updated. (c) The head of `web/e2e/messages.e2e.ts` still says the page of the message of the week only, and "the tests with a message fix the answers of the server", which the committee tests further down do not. Checked: read of the three places.
  - 2026-10-31: Fixed: the comments are true again.

**R2 – The empty text of the committee page has no test.** "No message: no team chosen." (R3 of the readings) appears in no test: search for `no team chosen` in `web/e2e` has no hit, while `/messages/week` has its empty text tested. The server side (`null`) is tested. The start page has no button without a team, so only an old link reaches it; the risk is a wrong prop on one page. The tests of step 3 did not ask for it. Checked: search.
  - 2026-10-31: Fixed: a browser test with a fixed `null` answer shows the page text.

**R3 – The join of club and team is in the server.** `api/messages.rs:76` makes the name of the heading (`format!("{} {}", club.name, team.name)`) and the core takes it as `team_name: &str`. Decision 0024 says the server only passes in the stored games, their places and the names, and `week_message` takes the club and the team. So R2 of the readings is not fixed by a core test: the core tests use the finished string, and only `tests/messages.rs` fixes the join. It is what `technical.md` asked for, and one line. Better for story E, which needs headings too: pass the club and the team. Checked: read, search for `club.name` in `crates`: this is the only join in Rust.
  - 2026-10-31: Fixed: `committee_message` takes the club and the team and joins them in the core.

**R4 – The handlers are the same five lines.** `week` and `committee` in `api/messages.rs` differ only in the last call, and story E adds two more. The reason `technical.md` gives for one page component (four pages with the same behaviour after story E) holds here too, and the risk is real: a future handler with another season or `today` than its flag on the start page would break "the button and the message agree" (decision 0024). No fault now. Checked: read.
  - 2026-10-31: Fixed: the routes share `message_view`.

**R5 – For Close: living documents this change makes false.** Normal for this stage, listed so that none is missed. `docs/product/messages.md` ("Now: the message of the week", the part "Messages", the server address); `docs/product/start-page.md` lines 12, 24 and 33 ("Otherwise the part is not shown"; with a team the part now always shows); `docs/product/duties.md:50` ("come in the next stories"); `docs/architecture.md` (`messages` in the core, `/messages/committee` and `MessagePage.svelte` in `web/`, the section "Messages"). Decision 0025 says a past open place alone "gives no message and no button": that is about the message of the week; the committee message differs on purpose (R1 of the readings), which only this file records. Checked: search for `messages/week` and `committee` in `docs/`.
  - 2026-10-31: For Close.

### Acceptance review
Acceptance review, 2026-10-31, by a fresh agent. The arguments held only the mode and the change
folder; nothing was ignored. Read: `change.md`, story D and its context in the epic's
`functional.md` with the epic's `idea.md`, `docs/vision.md` (journey 5), `docs/product/messages.md`,
then the diff; the **Deviations** only after that.

**Verdict: accepted with notes.** What was built is what was meant: one Dutch text for the referee
committee, with the team's table and referee places of this week and full names, from a button that
is there in every week. No concerning deviation. Next: `/close`; A1 to A3 need no change.

**Runs:** `cargo test -p team_core messages` 28 passed; `cargo test -p team --test messages --test message_logs`, the last 7 shown passed; `npm run build`, then `npx playwright test messages` 124 passed (the file with the projects it depends on), on the real server with the fake game list. Screen read in those tests only; not on a phone. `followed_teams` in the local database is empty, so no real club was tried.

| Criterion of story D | Built | Checked |
|---|---|---|
| "Committee message" whenever a team is chosen, also a week without places; opens this week | Yes, below "Next", after "Message of the week"; without a team no button | run: browser test on the real routes, `tests/messages.rs` |
| Team and week, then each game in time order, first and last names, "nog N open", no line for a kind without a place, no other club teams | Yes, the spec's example character for character | run: core test; trace: `schedule.past` keeps time order within a date (`crates/core/src/schedule.rs:99`), the stable sort on the date keeps it |
| A week without places: heading and "Geen …", "Copy" works | Yes | run: browser test reads the clipboard |
| Past days stay: who did it, or "niemand", also a place that stayed open | Yes | run: core test |
| Copying as in story C | Yes, one shared page | run: refused clipboard tested on `/messages/week`, the same component; committee page not separately |
| No last name: first name; a coming game gone or no longer a home game is left out | Yes | run: core tests |

**Points the spec left open:**

**A1 – Faithful adaptation: the open forms.** Only a past place that stayed open: the game with "niemand", not "Geen …" (reading R1, confirmed by the coordinator; it follows criterion 4, and "Geen" would be false). A start time not known: "za 31 okt, tijd nog niet bekend, …", as story C (shared `duty_lines`; read, no test for this message). Without a team: no button, and on an old link "No message: no team chosen." These serve the user. Checked: trace and run, as noted.
  - 2026-10-31: No action needed.

**A2 – Neutral drift: the heading joins the club and the short team name.** The heading is the club name plus `short_name` (`crates/core/src/team_choice.rs:83`). When the game list's team name does not start with the club name, the short name stays whole and the heading doubles it ("Blue Drakes Blue Drakes-X M16 1"); a source form "Blue Drakes - M16 1" comes out without the dash. The start page has the same join, so the coordinator would already see it, and can edit the text after pasting. Not checked on real data (no chosen team locally). Worth one look at the first real message.
  - 2026-10-31: No action needed: look at the heading in the first real message.

**A3 – Neutral drift: two limits the spec accepts or does not name.** Two players with the same full name are not told apart (reading R5): harmless for the committee. In a week across the change of season, the text says "Geen …" for the new team while the old season's places of that week are not read (`games_of` of the current season, as the documented limit of story C). The summer has no such games as far as the spec tells; not checked. Sunday evening cannot announce next week's Monday: accepted by the coordinator (S25), and the idea line in `docs/ideas.md` is still open.
  - 2026-10-31: No action needed: the coordinator accepted these limits.

**Deviations:** "None from `functional.md`" is true. The signature of `committee_message` in step 1 is changed by R3 of the build review, recorded there; not visible to the user.
