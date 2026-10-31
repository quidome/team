# Technical spec: The committee message

**Status**: approved
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
- [ ] Step 1
- [ ] Step 2
- [ ] Step 3
- [ ] All checks pass

## Deviations
None from `functional.md`. Steps 1 to 3 go into one commit: step 2 changes the generated front-end types.

## Review findings
