# Idea: Duties

**Status**: ready for discovery
**Vision fit**: Priorities 3 ("duties and the message for the next game") and 4 ("the message for the referee committee"); journeys 3, 4 and 5; core value 2 ("the division of duties is visible and fair"). After it, the sheet is no longer needed for duties and the weekly messages.
**Date**: 2026-10-18

## The problem
Each week the coordinator keeps in the sheet who drives to the team's away games and which players
fill the team's jury and referee places at the home games of other club teams, writes the message
for the next game and the message for the referee committee by hand, and keeps a tally so the work
is shared. Both kinds of duty cost about the same time (coordinator, 2026-10-18).

## The idea
An epic of four stories, in this order (coordinator, 2026-10-18):

| Story | What | Replaces in the sheet |
|---|---|---|
| A | **Duty places**: the number of drivers at the team's away games; the team's jury and referee places at the club's home games (other club teams), as the club's list gives them | the list of places |
| B | **Who does it**: a player on each duty place; after the game, done or someone else; the duty total of each player | the names and the tally |
| C | **The message for the next game** (journey 4): time, place, who drives, who does jury or referee duty | the weekly message |
| D | **The message for the referee committee** (journey 5): each game, its start time, and which player (first and last name) does which duty | the weekend message |

## Why now
Games (v0.2.0) and attendance at training (v0.3.0) are built; duties are the next priority of the
vision and the second thing the coordinator does every week. Attendance at games is not needed
first: the coordinator sets the number of drivers (coordinator, 2026-10-18).

## How it fits
- The schedule already shows the team's games and the club's home games of other club teams
  ("possible duty games"); the duty places are added to those games.
- Players and their roles exist (attendance at training): only players with the role "plays
  games" on the date of the game get duties (coordinator, 2026-10-18).
- The list of changes already shows when a game moves; a moved or gone game with duties matters.
- Principles: duties go to players, even when a parent drives; the tool only produces text to
  copy; one user changes data.

## User view
The coordinator gets the club's list (it says which team covers which jury and referee places at
which home game) and copies the team's part: at a possible duty game in the schedule, "2 jury
places, 1 referee place". At each away game the coordinator sets the number of drivers. The team
agrees in the chat who does what; the coordinator records the player on each duty place. When
choosing, the tool shows each player's duty total next to the name; it does not propose a player
(coordinator, 2026-10-18: "show totals only"). After the game the coordinator confirms who did it
or records someone else; that counts in the totals. Around training the coordinator copies the
message for the next game; before the weekend the message for the referee committee.

## Open questions
For `/discover`:
- The club's list: how often it comes, what it names for each place (date, time, game, jury or referee), and whether a place can change after it is entered.
- Drivers: is the number of drivers per away game, or per car with seats? Is a "departure time" (the old scenarios) needed in the message?
- The duty total: one number, or per kind (drive, jury, referee)? Only this season? Do duties of players who left the team stay in the totals?
- "Done or someone else" after the game: is a place that nobody did a "not done" with a reason, like absences?
- A game that moves or leaves the game list after its duties are set (idea of 2026-10-13: what a gone game means for its duties and the totals).
- The language of the messages (idea of 2026-10-10: messages from templates, in a language of the coordinator's choice; the screens stay in English).
- Is "possible duty games" still the right phrase now that "duty place" exists (open question in the vision)?
- What the message for the referee committee must contain exactly, and whether it is one message for a weekend or for each game.

## Change of 2026-10-25: duty places given to club teams
From the reviews of story A (`/vision idea: duty places given to club teams from the schedule of
the referee committee`). This replaces the table and referee part of the idea above ("copies the
team's part"); drivers stay as they are.

- **The problem.** The referee committee gives the places to club teams, not to players. Today the
  coordinator reads its schedule and works out "our team has 1 referee and 1 table place". That
  step is extra work and can go wrong. A place with no team in the tool also cannot follow the
  team when another team is chosen (review finding R1 of story A).
- **The idea.** The coordinator copies the schedule of the referee committee as it is. Every home
  game of the club, also a home game of the team, has the same 4 duty places: referee 1, referee 2,
  table 1 and table 2. Each place gets a club team from a pull-down list, or stays empty. Up to 4
  different teams at one game; the two table places usually go to one team. The tool does not name
  the tasks at the table (the players arrange that themselves). No typing.
- **Handled.** A home game is handled when the coordinator taps a button such as "This game is ok
  now", also with empty places. Until then it is an open item.
- **The team's places.** The places given to the team are the team's duty places. In story B the
  coordinator puts a player of the team on each of them.
- **Not now.** Reading the committee's sheet into the tool (it is an untidy sheet; entry by hand
  with pull-down lists is better for now).
- **Words.** "Jury" becomes "table"; "duty place" and "referee committee" have a new meaning
  (`docs/language.md`, 2026-10-25).
- The idea of 2026-09-27 in `ideas.md` ("Club duty allocation") is the same idea; its line is removed.

For `/discover`: which club teams the pull-down list offers (the teams of the club in the game
list of that season?); what a game shows when a place's team leaves the game list; whether "ok"
must be tapped again after a change of the game or of a place; what the start page and "Season"
show for home games where the team has no place; the message for the referee committee with all
places, or only the team's.
