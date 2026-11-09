# Functional spec: The report

**Status**: approved
**Change**: docs/changes/2026-11-09-the-report
**Date**: 2026-11-09

## Context
Now and then the coordinator must steer a conversation with a player, a parent or the team, and
needs facts for it. Sometimes people must be convinced that the effort or the duties are not in
balance. Today the numbers are on two pages made for entering data: "Attendance" (for each player,
how often there and absent at trainings and checked games, with valid and not valid absences) and
"Duties" (for each player, drive, table and referee as "done · planned"). Players cannot be
compared there at a glance, and nothing can be copied. The coordinator's sheet still does this job,
with attendance as percentages.

The report is one new page for the current season: a table with one row for each player
(attendance as percentages, duties as done and planned), the details of a player on a tap, and a
"Copy" for each column so the coordinator can put the numbers of one column into the chat. It adds
no new data: the numbers are the same as on "Attendance" and "Duties".

**Serves the purpose**: this is the purpose on one screen: what each player gives to the team, the
effort at training and games and the share of the duties (journey 8, priority 7 in `vision.md`).

## User stories

### Story A: The table
**As the** coordinator, **I want to** see one table with what each player gave this season,
**so that** I have the facts when I steer a conversation.

#### Acceptance criteria
- [ ] The start page has the link "Report" after "Duties" in its line of links: "Season · Attendance · Duties · Report · Team · Players · Settings".
- [ ] "Report" shows the current season only, with the season in the heading ("Report 2026-2027"); there is no season choice.
- [ ] The table has the columns name, Trainings, Games, Drive, Table, Referee, and one row for each player of the season (see "Which players"), sorted by first name, with the short names used on "Duties" ("Anna B." and "Anna V." when two players have the same first name; hidden players too).
- [ ] Players with a role today or later in the season are in the table (as on "Team", so a player whose role starts in December is in it); players who had a role earlier in the season and have none today or later are below it under "Left the team", in the same columns.
- [ ] Above the table, when the numbers are not complete: "2 training days not opened · 1 game not checked" (each part only when not 0). It counts the training days before today that were never opened (as on "Attendance") and the games that are the open item "not checked" on the start page. A tap on a part opens the oldest of them; "Back" there returns to "Report".
- [ ] Trainings and Games show the percentage "there" of the trainings or games that count for the player, rounded to a whole number, half up (2 of 3: "67%"; 1 of 8: "13%"). A valid absence lowers the percentage as a not valid one does.
- [ ] A cell where nothing counts for the player shows "–" (for example Games for a player who only trains, or Trainings before the first training).
- [ ] Drive, Table and Referee show "done + planned" ("1 + 2"; "0 + 0" when the player has the role "plays games" in the season and no duty of that kind). For a player who never had the role "plays games" in the season and has no duty, the three cells show "–".
- [ ] For each player, "there", "counted", "done" and "planned" are the same numbers as on "Attendance" and "Duties" of the current season at the same moment.
- [ ] With no row at all: "No trainings, games or duties yet this season."
- [ ] The table fits on a phone 360 pixels wide without scrolling to the side, with names of up to 10 characters.

#### Behaviour
The coordinator taps "Report" on the start page and sees, for example:

| | Trainings | Games | Drive | Table | Referee |
|---|---|---|---|---|---|
| Anna | 90% | 100% | 3 + 1 | 1 + 0 | 0 + 1 |
| Bram | 67% | 50% | 0 + 0 | 2 + 1 | 1 + 0 |
| Cas | 100% | – | – | – | – |

**Left the team**

| | Trainings | Games | Drive | Table | Referee |
|---|---|---|---|---|---|
| Dirk | 50% | 33% | 1 + 0 | 0 + 0 | 0 + 0 |

Cas only trains: no games and no duties. Dirk left in October; the numbers stay.

**Which players.** A player has a row when at least one of these holds in the current season: a
training or game counts for the player (the rows of "Attendance"), or the player has the role "plays
games" at some date in the season, or the player is on a duty place of the season (the rows of
"Duties"). Hidden players are included, as on both pages.

**What counts** follows the existing rules, without change:
- A training counts for a player when it exists (opened or added), is not cancelled, its date is today or earlier, and the player has a role on that date.
- A game counts when it is a game of the team, is checked, and is not "not played"; it counts for a player on its list with "there" or "absent". Extra players never count.
- "Absent, valid" and "absent, not valid" follow the setting of the reason now. An absence without a reason is not valid.
- A duty place counts as done from the start time of its game (by the player on it, or by the player of "Someone else"), and as planned before it; an open place and "Nobody" count for nobody. A not played game counts no duty, except its drives with "The drivers drove" on. The rest of "What counts" in `docs/product/duties.md` holds as it is.

#### Errors and edge cases
- Early in the season, only "–" and "0 + 0" for most players: that is correct; the page does not hide them.
- No team chosen for the season: the attendance columns work as on "Attendance"; no duty can exist, so all duty cells show "–" (not "0 + 0", which reads as "did no duties").
- The page cannot be loaded: "The report could not be loaded. Load the page again.", as on the start page.
- A player who changed role during the season (for example from "only trains" to "plays games") has one row, as on "Attendance".

### Story B: The details of a player
**As the** coordinator, **I want to** see what is behind the numbers of one player, **so that** I
can explain a low percentage or a difference in duties.

#### Acceptance criteria
- [ ] A tap on a name opens the details of that player under the row; a second tap closes them. More than one player can be open at the same time. This also works under "Left the team".
- [ ] Trainings: "9 trainings: 6 there (67%) · 2 absent, valid (22%) · 1 absent, not valid (11%)". Games: the same with "games". A line where nothing counts is not shown.
- [ ] Each percentage in the details is the number of that part divided by the number counted, rounded to a whole number, half up. The three percentages can add up to 99 or 101; the numbers are always exact.
- [ ] For each duty kind with at least one duty of the player: "Drive: 3 done · 1 planned".
- [ ] Below a kind, only when not 0: "1 done by another player" (planned for this player, done by another player with "Someone else"), "1 done for another player" (planned for another player, done by this player with "Someone else"), "1 not done" (planned for this player, corrected to "Nobody").
- [ ] The corrections count only at games whose duties count (the same rules as "done"), and only on the places of the team.
- [ ] A player with no line at all: "Nothing counts yet this season."
- [ ] A link "Missed trainings and games" opens "Attendance" of the current season with the list of that player already open (each missed training and game with its date and reason); "Back" there returns to "Report".

#### Behaviour
A tap on "Bram" shows:

```
Bram
  9 trainings: 6 there (67%) · 2 absent, valid (22%) · 1 absent, not valid (11%)
  4 games: 2 there (50%) · 1 absent, valid (25%) · 1 absent, not valid (25%)
  Table: 2 done · 1 planned
    1 done for another player
  Referee: 1 done · 0 planned
    1 done by another player
    1 not done
  Missed trainings and games
```

How the corrections relate to "done": a duty "done for another player" is in this player's "done"
(Bram did it). A duty "done by another player" or "not done" is not in this player's "done" (Bram
was planned, but did not do it). So the corrections explain the numbers; they are not added to them.

#### Errors and edge cases
- A correction at a game that was later marked "Not played" does not count (the game counts no duty), except drives with "The drivers drove" on.
- A place corrected back to its planned player (the planned player chosen again under "Someone else") has no correction: that undoes it.
- A correction where the player of "Someone else" left the team or is hidden still counts for that player, who has a row (the player is on a duty place of the season).

### Story C: Copy a column
**As the** coordinator, **I want to** copy one column as text for the chat, **so that** I can show
an imbalance.

#### Acceptance criteria
- [ ] Each column (Trainings, Games, Drive, Table, Referee) has "Copy". A tap puts the text on the clipboard and shows "Copied"; when the browser refuses, the text shows with "Select the text and copy it." and a tap on it selects all of it, as on the other message pages.
- [ ] The text is in Dutch: a heading with the kind, the team and the season, and "tot en met" today's date of the tool's clock; then one line for each player, sorted by first name, with the short name.
- [ ] Only players with a role today or later in the season are in the text (not "Left the team"). A player whose cell is "–" is left out.
- [ ] Trainings and Games give the percentage only. Drive, Table and Referee give the done number, with the planned number in brackets when it is not 0.
- [ ] The text holds no valid or not valid split, and no corrections.

#### Behaviour
```
Trainingen Blue Drakes M16 1, seizoen 2026-2027 (tot en met 9 nov)
Anna 90%
Bram 67%
Cas 100%
```
```
Rijden Blue Drakes M16 1, seizoen 2026-2027 (tot en met 9 nov)
Anna 3 (+1 gepland)
Bram 0
```
The kinds: "Trainingen", "Wedstrijden", "Rijden", "Tafel", "Scheidsrechter". Without a chosen team
the heading has no team: "Trainingen seizoen 2026-2027 (tot en met 9 nov)".

#### Errors and edge cases
- A column where every player in the team has "–": "Copy" is not shown for that column.
- The tool never sends the text: the coordinator carries it to the chat.

## Interface
- A new page "Report" (address `/report`), linked from the start page after "Duties".
- On a phone: the table with narrow columns; each column heading has a small "Copy" under it. The details open under the row of the player.
- Example screens: see the stories.

## Consistency
- The numbers are the numbers of "Attendance" and "Duties"; the rules in `docs/product/trainings-and-attendance.md` and `docs/product/duties.md` hold without change.
- "Left the team" works as on "Attendance".
- The short names work as on "Duties".
- "Copy" works as on the other message pages; the texts are Dutch, like the other messages.
- A tap that opens a part under a row works as on "Attendance".
- The words "done", "planned", "absent, valid", "absent, not valid", "Someone else" and "Nobody" are the words of the existing pages.

## What changes for existing users
A new page and a new link on the start page. "Attendance" gets one small addition: opened from
"Report", it shows the list of that player already open and a "Back" to "Report" (today it has no
links to other pages). Otherwise "Attendance" and "Duties" stay as they are. No data changes.

## Warnings from architect and engineer
- **One source for each total.** The duty totals and the attendance totals each have one source today. The report must use those sources, not count again, or the numbers drift apart after the next rule change.
- **The corrections are new counts.** "Done by another player", "done for another player" and "not done" are counted nowhere today. They must follow exactly the rules of the duty totals (not played games, "The drivers drove", the places of an earlier team after another team is chosen, games that left the game list).
- **Two sets of players.** The rows are the union of the rows of "Attendance" and of "Duties", which use different rules. The rule must be written in one place, so the pages cannot disagree about who is a player of the season.
- **One moment.** All numbers of the page must come from one reading, so a save on another screen cannot give a row whose numbers do not match each other.
- **Names.** The copied text holds the names of children with their attendance. It goes only to the clipboard; names never go into a log.
- **The date.** "tot en met 9 nov" uses the date of the tool's clock (the Netherlands), not of the phone.
- **The phone.** Five columns at 360 pixels are tight; the width must be checked with real names on a phone.

## Not in scope
- Sorting by a column, or any ranking of players.
- An earlier season, or another period than the season.
- A list of each missed event and each duty with its date inside the report (for trainings and games, the link to "Attendance" gives it).
- Other changes to "Attendance" or "Duties" than the "Back" of story B. After real use, decide if their totals are still needed (an idea for `docs/ideas.md` at close).
- Sharing the report with anyone but the coordinator, other than the copied text.

## Open questions
- None for `/vision`: all words exist. "Report" covers the table and the details.
- For `/close`: the line of links of the start page in `docs/product/trainings-and-attendance.md` lacks "Duties" (found by the spec review); fix it with the new link "Report".

## Answers to the spec review (2026-11-09, with the coordinator)
- Note 1 (incomplete numbers): the line "N training days not opened · N games not checked" above the table (story A).
- Note 2 (players whose role starts later): "today or later", as on "Team" (stories A and C).
- Note 3 (the detour to "Attendance"): the link opens "Attendance" with the player's list open, and "Back" returns (story B).
- Note 4 (copied attendance without the split): percentage only, as the coordinator chose. The coordinator is not sure and finds it easy to change later; a line in `docs/ideas.md` keeps the option (valid absences in brackets).
- Smaller points: without a chosen team the duty cells show "–" (story A); the out-of-date line of links goes to `/close`.

## Spec review
Reviewed 2026-11-09 by `/review spec`, in a fresh agent. Read: this spec, `idea.md`, `change.md`,
`docs/vision.md`, `docs/language.md`, and in `docs/product/` the pages trainings-and-attendance,
duties, start-page, messages, principles and players-and-team. `technical.md` was not read (there
is none yet).

**Serves the purpose: yes.** Checked against `vision.md` by reading it: the purpose sentence, core
values 1 and 2, journey 8, priority 7 and "Where we are" ("Effort: halfway" until the report). The
page puts what each player gives (attendance, with the valid split in the details, and the duties
done and planned) in one place, without ranking. That matches the word "contribution". It takes
over the job the sheet still does (attendance as percentages), which is the success measure in
"Where we are".

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| A: The table | A few times a season, before or during a conversation with a player, a parent or the team, the coordinator opens one page on the phone to compare players at a glance and have facts ready. | serves – note | (1) **The numbers may be incomplete without the page saying so.** A training day that was not opened does not count, and a game that was not checked does not count. "Attendance" shows "3 training days not opened" and "Games count once they are checked"; the report shows neither. `idea.md` (User view) says "the numbers must be right, because they are used to convince people". A coordinator who opens the report before a conversation should see that something is still open, for example one line above the table with the count of training days not opened and games not checked. Checked by reading trainings-and-attendance.md ("The totals", the line of training days not opened) and start-page.md (open items 6 to 8); whether the coordinator would want this was not checked. (2) **Players whose role starts later.** The spec says "players in the team today are in the table", but `players-and-team.md` lists "the players in the team today or later in the season". A player with a role from December only has a row (the role "plays games" at some date in the season) but is neither "in the team today" nor under "Left the team". Write "today or later", as on "Team". Checked by search (`today or later` in `docs/product/`). |
| B: The details of a player | The coordinator taps one or two players to explain a low percentage (illness or not) or a difference in duties (taken over, not done) before saying it out loud. | serves – note | The detail lines and the three corrections answer the question in journey 8 ("how many absences were valid and not valid, and how many duties someone else took over"). The spec says plainly that the corrections explain "done" and are not added to it. That stops a wrong reading. Note: the link "Missed trainings and games" goes to "Attendance", where the coordinator must find the same player again and tap them, and "Attendance" has no link back ("Attendance has no links to the other pages", trainings-and-attendance.md, Interface). In a conversation that is a detour. Design could open "Attendance" with that player's list already open, or say how the coordinator gets back to the report (the browser's back, with the open rows kept or not). Checked by reading trainings-and-attendance.md; the use during a conversation was not checked with the coordinator. |
| C: Copy a column | To show the team or the parents an imbalance, the coordinator copies the numbers of one column and pastes them into the chat. | serves – note | This fits the principles (the tool only produces text; the coordinator carries it to the chat) and core value 1 ("the coordinator may copy parts of it into the team chat"). Note: **attendance goes into the group chat without the valid split.** "Trainingen … Bram 67%" in a chat with all the parents puts a player who was ill at the same level as a player who stayed away, and that is a child's name in front of the whole team. Core value 1 is about "attendance, with the reason for each absence". The idea wanted the split in the details "so a low percentage from illness is not hidden in a formula", and the copied text loses exactly that. The coordinator may want this on purpose (a short text, with the context added by hand), but the spec does not say that the coordinator chose it. Confirm with the coordinator that the trainings and games copies stay percentage only. A choice that keeps the text short would be a count of valid absences in brackets ("Bram 67% (2 geldig afwezig)"). Checked by reading `vision.md` (core value 1) and `idea.md` (the details); the coordinator's wish was not checked. Duty copies ("Anna 3 (+1 gepland)") have no such problem. |

**Overall: serves – note.** Nothing conflicts with the vision. The three notes are for Discover to
settle with the coordinator or to hand to Design. None needs the spec to go back.

Smaller points, not about whether the spec serves the user (checked by reading):
- `trainings-and-attendance.md` still gives the line of links as "Season · Attendance · Team · Players · Settings", without "Duties". `start-page.md` has the right line. That is a stale living document, not a fault of this spec; Close can fix it.
- Story A, "Errors and edge cases", says the duty cells show "0 + 0" or "–" without a team. A player with the role "plays games" then shows "0 + 0", which reads as "did no duties" when no duty could exist. That is correct by the rule; it is listed only so Design does not miss it.
