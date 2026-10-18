# Functional spec: Duties

**Status**: approved
**Change**: [2026-10-18-duties](./)
**Date**: 2026-10-18

## Context
The coordinator runs one youth team of the club. Besides the games and the trainings, the team
has two kinds of **duty**, and today the coordinator keeps them in a Google Sheet and writes the
messages about them by hand:

- **Jury and referee duty.** Every few months (irregular, a few times per half year) the club's
  referee committee gives the **team** (not persons) a set of duty places at **home games of other
  teams of the club**, never at the team's own games: for each such game 1 or 2 referee places
  and 1 or 2 jury places (the table: the clock and the game administration on the tablet). The
  committee's list names each game. In October 2026 the coordinator knows all places until the
  end of the calendar year. The team then divides those places among its players; to make that
  fair, the coordinator must see who already did what.
- **Driving.** For each away game of the team, the team needs one car for every 3 players who go.
  Parents drive, but the duty is given to the player (a principle of the product).

The coordinator also writes two messages by hand: before each game, one for the team (time,
place, who drives, who has jury or referee duty); before each weekend, one for the referee
committee (which player does which duty at which game). The tool only produces text to copy; it
never sends (a principle of the product).

The tool already has (see `docs/product/`): the games of the club from the game list, with the
schedule of the team's games and the home games of other club teams; the players of the club and
the team of each season with each player's role ("plays games" or "only trains") from a date; the
list of changes to games; and attendance at training. Attendance at games does not exist.

This is an **epic of four stories**, built in this order (coordinator, 2026-10-18): A duty places,
B who does each duty place and the duty totals, C the message for the next game, D the message for
the referee committee. After it, the sheet is no longer needed for duties.

Facts from the coordinator (2026-10-18):
- Only players with the role "plays games" get duties.
- When choosing a player, the tool shows the duty totals; it never proposes a player.
- Fairness is judged for each kind apart: drive, jury, referee. The totals that help are per
  kind, for the current season, done and planned.
- A duty counts as done by the player on its duty place, unless the coordinator corrects it.
- The number of drivers is proposed (one per 3 players), and the coordinator can change it; a
  changed number stays.
- The messages are in Dutch; the screens stay in English.
- The jury and referee part of both messages covers the coming week (answer to spec review S7, S9).
- Hidden players are offered for duties like the others (S4).
- A driver proposal that drops by itself never removes a chosen driver (S1).
- A game whose date has passed was played: its duties stay done, also when it leaves the game list later (S2).

## User stories

### Story A: Duty places
**As the** coordinator, **I want to** record the duty places of the team at each game, **so that**
the tool knows what must be filled, without the sheet.

#### Acceptance criteria
- [ ] A tap on a game in the schedule opens the page of that game: date, start time, home team, away team, hall and town, and its duty places.
- [ ] At a home game of another club team (coming or past), the coordinator sets the number of referee places and of jury places, each 0, 1 or 2, and saves.
- [ ] At an away game of the team, the page shows the number of drivers. While the coordinator has not changed it, it is the proposal: one for every 3 players with the role "plays games" on the game's date, rounded up (10 players: 4 drivers; 0 players: 0), marked "proposed", and it follows the team when players join or leave. The coordinator can set another number (0 to 10); from then on that number stays, without the mark "proposed". "Use proposal" goes back to the proposal. When the proposal drops by itself (a role changes) below the driver places already filled, the number stays at the filled places, with "more than proposed"; nobody is removed.
- [ ] A home game of the team has no duty places.
- [ ] In the schedule, a game with duty places shows the mark "duties", so the coordinator sees which games of the committee's list are already entered.
- [ ] A number of places below the number of filled places is refused: "2 places are filled. Clear one first."
- [ ] The page "Duties" lists the coming games (today and later) that have duty places, nearest first, each with its places: filled with the player's first name, or "open" (for example "Referee: open · Jury: Anna, open"). A game whose places are all filled shows "all filled". Below, "Show past games" opens the past games of the season with duty places, newest first.
- [ ] The start page links to "Duties" in its line of links.

#### Behaviour
The coordinator gets the committee's list in the chat and walks through it: in the schedule
(filter "Possible duty games") a tap on the game, "Referee 1, Jury 2", save; the next game. For an
away game the page already shows "Drivers: 4 (proposed)". Places can be set for past games too,
to enter what already happened this season.

#### Errors and edge cases
- A game that moves (another date, time or hall) keeps its duty places and the players on them. Its change in the list of changes says "has duties" with the filled places (for example "Jury: Anna, open"), so the coordinator can tell the players and the committee.
- A coming game (today or later) that leaves the game list keeps its duty places while it is gone; on "Duties" and on its page it shows "no longer in the game list". Its duties do not count while it is gone. If it comes back, they count again.
- A game whose date has passed was played: its duties stay done, also when it leaves the game list later (for example when the source cleans up). Its page says "no longer in the game list".
- A game that stops being a home game of another club team (for example the home and away teams swap, or it becomes a game of the team) keeps its places, with "This game no longer needs duties of the team." on its page and on "Duties", and its places do not count. Setting the places to 0 removes them.
- Another team chosen in the middle of a season (rare): the duties of the old team stay stored, and the same rule applies: coming ones show "This game no longer needs duties of the team." and do not count; past ones count. Choosing the old team again brings them back (coordinator, 2026-10-18, in Design).
- A game of an earlier season can be opened (from the schedule of last season) but its duty places cannot be changed: "Duties of an earlier season cannot be changed."
- "Duties" with no games with duty places: "No duty places yet. Open a game in the schedule to add them."

### Story B: Who does each duty place, and the duty totals
**As the** coordinator, **I want to** put a player on each duty place and see what each player has
done, **so that** I can divide the places fairly and the totals are right.

#### Acceptance criteria
- [ ] On the page of a game, a tap on a duty place opens the choice of a player: the players with the role "plays games" on the game's date, hidden players too (hidden only keeps lists short, as at a training), sorted by first name. Next to each name, that player's totals for that kind in the current season: "2 done · 1 planned". The tool marks no player as proposed.
- [ ] A tap on a player puts the player on the duty place and saves at once. "Clear" makes the place open again.
- [ ] A player who already has a duty place at the same game is not offered for another place at that game.
- [ ] A filled duty place of a game before today (Dutch date) counts as **done** by its player. A filled duty place of today or later counts as **planned**. Open places count for nobody.
- [ ] On a past game, the coordinator can correct a duty place: another player ("Someone else": the same choice), or "Nobody" (the place was not done; it counts for nobody and shows "not done (was Anna)", so the planned player stays visible). The correction replaces what counts.
- [ ] The page "Duties" has a table "Totals": one row for each player with the role "plays games" today or earlier in the current season, sorted by first name, and the columns Drive, Jury and Referee, each "done · planned" (for example "3 · 1"). "Done" is the duty total of the word list; "planned" is shown apart from it. Players who left the team stay in the table while they have duties in the season; hidden players have rows like the others.
- [ ] What counts: duties of coming games that are in the game list and still need duties of the team, and duties of past games (also when they left the game list later); see story A.
- [ ] Two players with the same first name show the first letter of the last name in the choice and in the message for the next game ("Anna B.", "Anna V.").

#### Behaviour
When a new set of places from the committee is entered, the coordinator opens "Duties", sees the
open places of the coming games, and for each place taps a player, looking at the totals next to
the names. The team usually agrees in the chat first; the coordinator records the result. After a
weekend nothing needs to be done unless something went otherwise: then a tap on the place,
"Someone else" or "Nobody".

#### Errors and edge cases
- A player is no longer offered for duty places on dates when the player has no role "plays games" (left the team, or only trains). A player already on such a place stays on it, with "Not in the team on this date" next to the name; the coordinator can clear or change it.
- A player can be deleted only when the player has no absences (existing rule) and no duties; else "Delete player" is not offered (as now for absences).
- Two screens: the last save wins. A duty place never holds two players.
- A save that fails shows "Not saved. Try again." and the place shows what is stored.

### Story C: The message for the next game
**As the** coordinator, **I want to** copy one message about the team's next game, **so that** the
team and the parents know when and where, and who drives or has jury or referee duty.

#### Acceptance criteria
- [ ] The start page shows "Message for the next game" when the team has a coming game (today or later), or jury or referee duties in the coming week. A tap opens the message: Dutch text, with a "Copy" button that puts it on the clipboard and says "Copied".
- [ ] The message has the date and start time of the team's next game, home ("Thuis") or away ("Uit"), the opponent, the hall and the town; for an away game the drivers by first name, with open places as "nog N open"; and the jury and referee duties of the team in the coming week (from today until the next Sunday, today itself when it is a Sunday), whether or not the team plays that week, each with the date, start time, the game, and the players by first name.
- [ ] A start time not known yet shows "tijd nog niet bekend".
- [ ] The text is built from what is stored at that moment; the coordinator may edit it after pasting in the chat.

#### Behaviour
Example for an away game:

```
Volgende wedstrijd: zaterdag 31 oktober, 14:30
Uit tegen B.Z. '72 M16 1
Sporthal Bulwijk, Woerden

Rijden: Anna, Bram, Cas (nog 1 open)

Jury en scheidsrechter deze week:
za 31 okt 16:00 Blue Drakes X12 1 – De Dunckers X12 1
  Jury: Eva, Fleur
  Scheidsrechter: Gijs
```

For a home game the line "Rijden" is left out. Without jury or referee duties in the coming week,
the part "Jury en scheidsrechter deze week" is left out. Without a coming game of the team (a free
week, or after the last game), the message has only that part.

#### Errors and edge cases
- No coming game of the team and no duties in the coming week: the button is not shown.
- The next game is no longer in the game list: the message is for the next game that is.
- Copying fails (the browser refuses): the text stays shown and selectable, with "Select the text and copy it."

### Story D: The message for the referee committee
**As the** coordinator, **I want to** copy one message with the team's jury and referee duties of
the coming week, **so that** the referee committee knows which player does which duty.

#### Acceptance criteria
- [ ] On "Duties", "Message for the referee committee" opens the message for the coming week: Monday to Sunday of the week of the coming weekend (on a Saturday or Sunday: this week), so duties on a weekday are in it too. "Next week" and "Previous week" move one week.
- [ ] The message lists each game of that week with jury or referee places of the team, in time order: the date, start time, home team – away team, then for each kind the players by **first and last name**; open places show "open" ("nog open").
- [ ] A week without duties: "Geen jury- of scheidsrechtersdiensten deze week." and the button "Copy" still works.
- [ ] Copying works as in story C.

#### Behaviour
Example:

```
Jury en scheidsrechters Blue Drakes M16 1, week 26 oktober – 1 november

za 31 okt 13:00 Blue Drakes X12 1 – De Dunckers X12 1
  Scheidsrechter: Eva de Vries
  Jury: Fleur Jansen, Gijs Bakker

zo 1 nov 11:00 Blue Drakes M14 2 – Lions M14 1
  Scheidsrechter: nog open
  Jury: Hugo Smit
```

#### Errors and edge cases
- A player with no last name shows the first name only.
- A game that is no longer in the game list, or no longer needs duties of the team, is left out of the message.
- Copying fails: as in story C.

## Interface
- Screens: the page of a game (new; from a tap on a game in the schedule, on "Duties", and in the list of changes); "Duties" (new; the coming and past games with duty places, the totals, and "Message for the referee committee"); the schedule gets the mark "duties" on games with duty places; the start page gets "Message for the next game" and the link "Duties" in its line of links ("Attendance · Duties · Team · Players · Settings").
- The messages are Dutch text shown on a page, with a "Copy" button.
- Server addresses follow in the design.

## Consistency
- Duty places are added to the games the schedule already shows; the words of the schedule stay ("Games of the team", "Possible duty games").
- The choice of a player works like marking at a training: a tap saves at once, "Not saved. Try again." when it fails, the stored state shown again.
- Players: only the role "plays games" on the date counts, as roles decide who is at a training; hidden players are not offered, as hidden reasons are not offered.
- Totals are worked out each time from what is stored, as the attendance totals are; nothing is counted on the side.
- The list of changes keeps its behaviour; a change to a game with duties also says "has duties".

## What changes for existing users
- New screens and a new link on the start page; nothing existing changes.
- "Delete player" is also not offered for a player with duties.
- No data to move: there are no duties yet. The coordinator enters this season's duties (also past ones) by hand.

## Warnings from architect and engineer
- **Games move and disappear.** Duty places are entered weeks or months ahead; the game list can move a game, swap home and away, change the team, or drop the game in that time. Every such case must keep the duty places and say clearly what happened (story A), and the totals must never count a duty of a game that is gone or no longer needs duties.
- **One count everywhere.** The totals next to the names (story B), the table on "Duties", and later an overview for the parents must give the same numbers from the same rule (done before today, planned from today, Dutch date; corrections replace; coming games that are gone or no longer need duties do not count; past games count, also when gone).
- **The driver proposal moves.** While not changed, it follows the players with the role "plays games" on that date; a role change can change the number after drivers were chosen. Filled places above a lower proposal must not disappear (story A: the number stays at the filled places, "more than proposed").
- **Two screens.** One coordinator, but a phone and a computer can save the same duty place; one place never holds two players, and a choice must not be lost without a sign.
- **Dates, weeks and weekends** follow Dutch time, as the schedule does.

## Not in scope
- Attendance at games (it may later make the driver proposal more exact).
- A departure time for away games in the message (the coordinator adds it by hand when needed).
- The overview for the parents with duty totals (journey 6; later).
- Recording which team of the club covers which places (the club's own division; idea of 2026-09-27).
- Proposing a player for a duty place (the coordinator chooses; coordinator, 2026-10-18).
- Messages in other languages, and templates the coordinator can edit (idea of 2026-10-10).
- Duties at the team's own home games, and duties other than drive, jury and referee.
- Totals over more than one season.

## Open questions
- For `/vision`: a word for the set of duty places the referee committee gives the team for a period (the coordinator says "shifts" and "a wave"); this spec writes it out in plain words.
- For `/vision`: the phrase "possible duty games" in the schedule, now that "duty place" exists (open question in the vision).

## Spec review
<Filled in by `/review spec` (a fresh agent). One row per user story. Design does not start
without this table. A "does not serve" row sends the spec back to Discover.>

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| A Duty places | Copying the committee's list (a few times per half year) and the number of drivers into the tool, so the sheet's list of places is no longer needed | serves – note | S1, S2, S3 |
| B Who does it, totals | Recording the player the chat agreed on, with the totals in view so the division stays fair; correcting after a weekend that went otherwise | serves – note | S4, S5, S6 |
| C Message for the next game | Copying, around training, the one weekly message the team and parents read: when, where, who drives, who has jury or referee duty | serves – note | S7, S8 |
| D Message for the referee committee | Copying, before the weekend, the full names per duty for the committee | serves – note | S9 |

Reviewed 2026-10-18 by a fresh agent (`/review spec`), against `docs/vision.md`,
`docs/language.md`, `idea.md` and `docs/product/` (schedule, list of changes, players and team,
trainings and attendance, read the game list). The arguments held only the mode and the change
folder; nothing was ignored. No story conflicts with the vision: duties go to players, the tool
only makes text, one coordinator, and stories C and D are journeys 4 and 5 almost word for word.
The notes are gaps for Discover to close (S7 first), not reasons to send the spec back.

- **S1 – The driver proposal drops below the filled places.** The warnings say filled places above a lower proposal "must not disappear silently", but no criterion says what the page shows then. "Refused while filled" fits a number the coordinator types, not a proposal that changes by itself when a role changes on the Team page. The coordinator needs a defined state (for example the proposal stays at the filled number, or "4 filled, proposal 3"). Checked: trace of story A criteria against the warning "The driver proposal moves".
  - 2026-10-18: Answered (coordinator): the number stays at the filled driver places, with "more than proposed"; nobody is removed (story A).
- **S2 – A past game that leaves the game list takes its done duties out of the totals.** Story A says a gone game's duties do not count while it is gone. For a past game the drive or jury duty did happen; if the source drops or renumbers past games, the totals silently lose real work, and core value 2 (anyone can check the division) suffers. The list of changes does not keep changes to past games, so the coordinator is not told either. Idea of 2026-10-13 is exactly this question; the spec answers it for coming games but the same rule hits past ones. Checked: search `docs/product/list-of-changes.md` ("A change to a past game is not kept") and `docs/ideas.md` (2026-10-13).
  - 2026-10-18: Answered (coordinator): a past game was played; its duties stay done, also when it leaves the game list later (story A and B, warnings).
- **S3 – The schedule does not show which games already have duty places.** The coordinator walks the committee's list through the schedule filter "Possible duty games", but the spec gives the schedule no mark for a game with places, so checking "did I already enter this one?" needs a tap on each game or a switch to "Duties". Small; a mark in the schedule row would serve the walk-through. Checked: trace of story A "Behaviour" and the Interface section (only the page of a game and "Duties" show places).
  - 2026-10-18: Fixed: the schedule shows the mark "duties" on games with duty places (story A, interface).
- **S4 – Hidden players in the team are left out of the choice, unlike the rest of the product.** `players-and-team.md` says hidden "only keeps lists short; it is not a status ... a hidden player in the team stays in the team", and hidden players in the team are marked at trainings. Story B instead follows hidden reasons. A hidden player with the role "plays games" then never gets a duty and the division looks fair while it is not. Either offer them (like trainings) or say why duties differ. Also unclear: does such a player have a row in "Totals"? Checked: search `docs/product/players-and-team.md` and `trainings-and-attendance.md` for "hidden".
  - 2026-10-18: Answered (coordinator): hidden players are offered like the others and have rows in "Totals" (story B).
- **S5 – "Nobody" keeps no trace of who was planned.** A place corrected to "Nobody" counts for nobody, so a player who did not show up looks the same as one who never had a duty. For fairness the coordinator may want to see that (the idea asked "a not done with a reason, like absences?"). The spec's answer is plausible and simple; it is not marked as a fact from the coordinator. Confirm with the coordinator. Checked: compare idea.md open questions with the spec's "Facts from the coordinator".
  - 2026-10-18: Fixed: "Nobody" shows "not done (was Anna)" (story B); no reason, unlike absences.
- **S6 – "Duty total" in the word list means duties done; the spec adds "planned".** Showing planned next to done is useful for dividing, but the word list's meaning ("how many duties a player has done") no longer covers what the totals show. A word question for `/vision`, not a gap in the story. Checked: search `docs/language.md` for "duty total".
  - 2026-10-18: "Duty total" keeps its meaning (done); the table shows "planned" apart from it (story B). No word change needed.
- **S7 – Duties on a weekend without a game of the team never reach the team.** The jury and referee part of the message is tied to the weekend of the team's next game. In a weekend when the team does not play (a free weekend, a holiday weekend, after its last game of the season) but has jury or referee places, no message names those players; with no coming game the button is not shown at all. Likewise a weekday game of the team pushes that weekend's duties out until it has passed. The coordinator then writes that message by hand again, against the north star ("every message ... comes from the tool"). Discover should say what the team message covers when the next game and the next duties are not in the same weekend. Checked: trace of story C criteria and edge cases.
  - 2026-10-18: Answered (coordinator): the jury and referee part covers the coming week (today until the next Sunday), whether or not the team plays; without a coming game the message has only that part (story C).
- **S8 – First names only can be ambiguous.** Two players with the same first name (allowed, with a warning, in `players-and-team.md`) look the same in the team message and in the choice of a player. Minor; the coordinator may add a letter by hand after pasting. Checked: search `docs/product/players-and-team.md` ("same first and last name").
  - 2026-10-18: Fixed: two players with the same first name show the first letter of the last name (story B, C); the committee message has full names.
- **S9 – Which games the committee message leaves out is half stated.** It leaves out games no longer in the game list, but says nothing about a game that "no longer needs duties of the team" (story A), and nothing about duties on a weekday, which then appear in no message at all (also not in story C). Checked: trace of story D edge cases against story A.

  - 2026-10-18: Fixed: the committee message covers Monday to Sunday of the coming week (also weekday duties) and leaves out games that are gone or no longer need duties (story D).
