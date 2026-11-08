# Functional spec: Attendance at games

**Status**: approved
**Change**: [2026-10-31-attendance-at-games](./)
**Date**: 2026-10-31

## Context
The coordinator runs one youth team and announces each week in the chat. Players then say in the
chat that they come to the game or not. The answers arrive over several days, so the coordinator
counts them by hand. Two mistakes cost the most: seeing too late that fewer than 7 players come,
when there is no time left to find extra players from other teams; and missing that a player who
must drive to an away game does not come.

This change gives each game of the team a list of who comes, who does not, and who has not answered.
The coordinator keeps it up to date during the week. The tool warns in time. After the game the
same list says who was there, which also serves the later report of what each player did.

The goals, in order (coordinator, 2026-10-31): first the warning in time, second the record after
the game. The user experience must be simple: from the start page to a saved answer in two taps,
with one hand on a phone. The number of requests is no limit on the design (coordinator).

How attendance at training works now: everyone is "there" at first, the coordinator marks only
absences with a reason (required), each tap saves at once, and "Checked" confirms the training. A
past training that is not checked is an open item on the start page. A game differs: before a game,
"there" at first would hide who has not answered, so a shortage would not show.

Facts from the coordinator (2026-10-31):
- Fewer than 7 players coming is a problem. The number is fixed in the tool, for the whole team, with no setting.
- The warning starts 3 days before the game: on Wednesday for a game on Saturday, on Thursday for a game on Sunday. It holds for home games and away games.
- When there are too few players, the coordinator finds extra players from other teams.
- A reason for not coming is optional: the chat often gives none.

## User stories

### Story A: Who comes to a game
**As the** coordinator, **I want to** mark during the week who comes to a game of the team, **so
that** I see at once how many players I have and whom I must still ask.

#### Acceptance criteria
- [ ] The page of a game of the team has at the top a list of the players with the role "plays games" on the date of the game, also hidden players. Players who only train are not on it.
- [ ] Each player is in one of three states: "coming", "not coming" (with a reason or without one), or "no answer". A player starts as "no answer".
- [ ] Each row has the buttons "Coming" and "Not coming". One tap saves the state at once. A tap on the state that is already chosen goes back to "no answer", except on a checked game (story C).
- [ ] From the start page, a tap on a game in "Next" opens the page, and a tap on a button saves: two taps.
- [ ] A row with "not coming" or "absent" has the link "Reason", which opens the reasons that are not hidden, at any moment, also on a checked game. They do not open by themselves. One tap adds or changes the reason; a tap on the chosen reason removes it. The reasons close at a tap on another row. The reason stays when the state goes to "Coming" and back. An absence without a reason counts as not valid.
- [ ] The list has the groups "No answer", "Coming" and "Not coming", in that order, each sorted by first name. A group without players is not shown. Rows do not move while the page stays open, also after a return from the chat; the groups sort again when the page is opened.
- [ ] Above the list is the count, which stays in view while the list scrolls: "6 coming · 1 not coming · 2 no answer". A part with 0 is left out. "Coming" is the players coming plus the extra players, the same number as in "6 of 7".
- [ ] Below the list is "Extra players" with [−] and [+], from 0 to 10, saved at each tap. It is for anyone who is not on the list: a player of another team, or a player who only trains. Extra players have no names.
- [ ] A coming game with fewer than 7, counting players coming plus extra players, shows "Too few players: 6 of 7" above the count, from 3 days before the game until its start time, and earlier once the game has an answer. From 7 on it is not shown. The rule of "too few" is the same as for the open item; only the open item waits for the 3 days.
- [ ] The row of a coming game of the team in "Next" shows "6 coming · 2 no answer" (players coming plus extra players, and no answer). On "Season" the row shows it only when the game has at least one answer or extra player. A part with 0 is left out, so a game with no answers in "Next" shows "9 no answer".
- [ ] The messages and the duty overviews do not change, except that a not played game leaves the week messages (story E).

#### Behaviour
```
Sat 31 Oct · 14:30 · Uit tegen B.Z. '72 M16 1
Too few players: 6 of 7
6 coming · 1 not coming · 2 no answer

No answer
  Anna           [Coming] [Not coming]
  Bram           [Coming] [Not coming]
Coming
  Cas            [Coming ✓] [Not coming]
  ... 4 more
Not coming
  Eva · Ill      [Coming] [Not coming ✓]

Extra players  [−] 1 [+]
```

The rest of the page of a game (places, drivers, "Schedule entered") stays below the list.

#### Errors and edge cases
- A tap that cannot be saved: the row shows the stored state and the page says "Not saved. Try again.", as at a training. A state never looks saved when it was not.
- Two screens: the last tap wins. Taps for one player are sent one after the other, so a late answer never overwrites a newer tap.
- No player has the role "plays games" on the date: "No players on this date." "Extra players" still works.
- All reasons hidden: "Not coming" works without a reason.
- A game moves to another date: the answers stay. The list follows the roles on the new date, and the 3 days of story B follow the new date. When the game has answers, the page says "Answers from before the move to Wed 4 Nov." with the button "Answers checked". Only that tap ends it; "Seen" in the list of changes does not. From 3 days before the new date, an unconfirmed move is an open item: "Game Wed 4 Nov moved: check the answers".
- A game leaves the game list: the answers stay, and the page of the game stays reachable while it has answers or duties. A coming game that left gives no warning and no count.
- A role changes: a stored answer stays, but a player not on the list of that date is not shown and does not count.
- A game of an earlier season can be marked as well.
- Games before the release are not entered as a rule: the totals start at the release, and "Attendance" says so (story D).
- "Delete player" is refused for a player with an answer at a game, as for a player with absences or duties; such a player can be hidden.
- Games that are only home games of other club teams have no list: their players are not playing. An away game of the team in the club's own hall has the list, as each game of the team.

### Story B: Warnings in time
**As the** coordinator, **I want to** be told in time when a game has too few players or a driver
does not come, **so that** I can act before the game.

#### Acceptance criteria
- [ ] From 3 days before the date of the game until its start time (with no known start time: until the end of that date), a game of the team with fewer than 7 players (coming plus extra) that is not checked is an open item: "Game Sat 31 Oct: 5 of 7 players, 3 no answer". Without "no answer": "Game Sat 31 Oct: 5 of 7 players". For more games: "2 games: too few players", and a tap opens the list of those games below the item.
- [ ] A coming game with a player on a driver place who is "not coming" is an open item, at once, also more than 3 days before the game, until the start time of the game: "Away game Sat 31 Oct: driver Anna is not coming". With two such drivers at one game: "Away game Sat 31 Oct: 2 drivers are not coming". For more games: "2 games: a driver is not coming", with a list as above.
- [ ] A tap on an item for one game opens the page of that game. "Back" returns to the start page.
- [ ] An item goes away by itself when its problem is gone: the 7 is reached, the player is "coming" or "no answer" again, the driver place is cleared or has another player, the game is checked, or the warning period has ended.
- [ ] The page of the game shows the same fact at the driver place: "Anna · not coming". On a past game: "Anna · absent", so the coordinator can use "Someone else" or "Nobody"; the tool does not correct the duty by itself.
- [ ] In the choice of a driver, a player who is "not coming" stays a choice, with the note "Not coming".
- [ ] The tool never removes a driver: what the coordinator stored does not change by itself.
- [ ] The open items are in this order: the game list; changes not seen; "too few players"; "driver not coming"; "moved: check the answers"; home games with the schedule not entered; games with open places; training days not opened; trainings not checked; games not checked.
- [ ] The rule of "too few" is one rule: the page of the game and the open item give the same answer.

#### Errors and edge cases
- A game that left the game list, or a game that is no longer a game of the team, gives no warning.
- A driver who is "no answer" gives no warning. A driver with a stored "not coming" who is no longer on the list of that date gives no warning.
- Table and referee places give no warning: the warning is for drivers, because a car that is missing affects who gets to the game.
- A game without a known start time: the date decides, as for all dates. Dates are Dutch dates.

### Story C: After the game
**As the** coordinator, **I want to** confirm with one tap who was there, **so that** the record is
correct with almost no work.

#### Acceptance criteria
- [ ] After the date of the game, the buttons say "There" and "Absent", and the groups "No answer", "There" and "Absent"; the count says "8 there · 1 absent". On the date itself the words stay "coming" and "not coming". The stored state does not change: "coming" is "there", "not coming" is "absent".
- [ ] A game of the team has the button "Checked" from its start time on; with no known start time, from the day after its date. Checked: the page shows "Checked" with "Undo check", and on "Season" and in "Last" the row shows "· Checked". A checked game shows only the count; a tap opens the list.
- [ ] A tap on "Checked" makes each "no answer" an "absent", without a reason, in one step, and says so: "2 players with no answer are absent."
- [ ] "Undo check" brings back to "no answer" only the players that "Checked" settled and that were not tapped after that. Players marked by hand keep their state.
- [ ] On a checked game a player can be changed between "there" and "absent"; a tap on the chosen state does nothing. A player who gets a role later shows as "no answer" on that game, does not count until the coordinator taps that player, and the page says so at the change of role, as at a training.
- [ ] A game of the team before today that is not checked, is not marked "not played", and did not leave the game list is an open item: "Game Sat 31 Oct not checked", for more "2 games not checked"; a tap opens the oldest. This holds for games from the day of the release on, in the same seasons as the trainings (the current season, and in July and August the season before). Older games are never an open item, but can be marked and checked.
- [ ] A game before its start time has no "Checked". On a game from before the release, the list is closed behind a tap.

#### Errors and edge cases
- A save that fails: "Not saved. Try again." and the stored state.
- Two screens: the last check or undo wins.
- A game with no player on its list can be checked.
- A past game that left the game list is no open item "not checked". While it has answers or duties, its page stays reachable, and the coordinator can tap "Not played" (story E).

### Story D: The totals on "Attendance"
**As the** coordinator, **I want to** see how many games each player was at, **so that** I see the
effort of each player, as with the trainings.

#### Acceptance criteria
- [ ] Each row of "Totals" on "Attendance" also shows the games: "of 9 games: 8 there · 1 absent, valid · 0 absent, not valid". A row with no game that counts leaves this part out.
- [ ] An absence without a reason counts as "absent, not valid". It has no tie to the reason named "No reason", which the coordinator can rename or hide.
- [ ] Only checked games that were played count, for a player who was on the list on that date with "there" or "absent". Extra players never count. "Attendance" says that the games start at the release.
- [ ] A tap on a player also lists the games the player missed, with the date and the reason.
- [ ] A player with games and no trainings has a row. The texts of "Attendance" that speak only of trainings also cover games, and "Left the team" also counts games.

### Story E: A game that was not played
**As the** coordinator, **I want to** say with one tap that a game was not played, **so that** nobody
gets a duty or an absence for it, also when I know it before the date.

Today a past game counts what is stored: a player on a place counts as done unless corrected, also
when the game left the game list. So the drivers of a game that did not happen count as having
driven, and the only way out is "Nobody" on each place. A game that left the game list before its date also starts to count again once its date has passed.

#### Acceptance criteria
- [ ] The page of each game of the team, coming or past, and of each other game with places of the team, has the button "Not played", below "Checked" when that is there. A game that left the game list has it too, while its page can be reached.
- [ ] One tap: the duties of the game stop counting in the duty totals (drive, table, referee; a drive still counts when "The drivers drove" is on), the attendance stops counting in the totals on "Attendance", the game gives no warning and is not an open item, and it leaves the messages and duty overviews. The places, players and answers stay stored.
- [ ] From its start time (without a start time: from the start of the game day), a not played game with driver places of the team shows the switch "The drivers drove", off at first. When it is on, the driver places count as at a played game (from the start time): done by the player on the place, with "Someone else" and "Nobody". When it is off, no drive of that game counts. The table and referee places and the attendance never count. "Undo" keeps the switch stored; it has effect again only after a new "Not played". (Coordinator, 2026-11-08: the drivers may have driven to an away game that was then not played.)
- [ ] Answers, extra players and duty places can still be changed on a not played game; the mark changes only what counts.
- [ ] The page shows "Not played" with "Undo" at the top, and the list is closed behind a tap. "Undo" saves at once and makes everything count again as before.
- [ ] On "Season", in "Next" and in "Last" the row shows "· Not played".
- [ ] A not played game does not block a player in the clash rule of another game.
- [ ] A not played game cannot be checked; a checked game can become not played, and "Undo" returns it to checked.
- [ ] A game that left the game list never becomes "not played" by itself: the list of changes keeps it until it is seen, and the coordinator opens the game and taps. (Coordinator, 2026-10-27: what is stored is never changed by a reading.)
- [ ] "Not played" and "Checked" hold only for the date on which they were tapped. When the game gets another date, it is a normal game of its new date again, with its answers and duties; nothing is deleted.

#### Errors and edge cases
- A save that fails: "Not saved. Try again." and the stored state.
- Two screens: the last tap wins.

## Interface
- **The page of a game**: the list, the count, "Extra players", from the start time "Checked".
- **"Next" and "Season"**: the count on the rows of coming games of the team; "· Checked" and "· Not played" on past games.
- **The page of a game**: "Not played".
- **The start page**: three new open items.
- **"Attendance"**: the games in "Totals".
- Server addresses follow in the design.

## Consistency
- One tap saves at once, with "Not saved. Try again.", as at a training.
- "Checked" and "Undo check" work as at a training, except that "Checked" also settles each "no answer".
- The reasons are the same reasons as at a training.
- The open items follow decision 0022: each comes from the rule of its source.
- Only the role "plays games" counts, as in the driver proposal.
- What the coordinator stored is never changed by a reading of the game list.

## What changes for existing users
- New: the list on the page of a game of the team, the count on rows, three kinds of open items, games in the totals, "Not played".
- Changed: the order of the open items; the duty totals leave out a not played game.
- No data to move: games before the release have no answers.
- The page of a game of the team gets longer: the list comes first, the places and drivers below it. A checked game shows only the count.

## Warnings from architect and engineer
- **A move and its note** must survive "Mark all as seen".
- **Three states.** Trainings store only absences (decision 0020); a game needs three states, so that rule does not carry over as it is.
- **"Undo check" must know** which players were settled by "Checked" and which were marked by hand.
- **One rule (decision 0022).** "Too few", "driver not coming" and "not checked" must each come from one rule, also on the page of the game.
- **A warning without a tap.** A driver warning can appear when a place changes, not only with an answer. A role change can change the count.
- **Two screens.** The last tap wins; a late answer must not overwrite a newer tap.
- **Dates** follow Dutch time.
- **Names.** Open items now hold a player name; logs never contain player names.
- **"Not played" touches released rules.** The totals, the choice of a player, the messages and "Season" read what counts; "Not played" must go through one rule, and it must not change what is stored.
- **A gone game must stay reachable** while it has answers or duties.

## Not in scope
- The driver proposal: a separate small line in `docs/backlog.md`. The tool does not use the answers to change the number of drivers.
- Names of extra players.
- Reading the poll from the chat, or sending anything to players.
- A quick way to enter games before the release ("All there").
- The report of what each player did.
- A setting for the number 7.
- Attendance of players at other games than games of the team.

## Open questions
None.

## Spec review
Fresh review, 2026-10-31. Read: this spec, `idea.md`, `docs/vision.md`, `docs/language.md`, all of
`docs/product/` except `login.md`, `choose-club-and-team.md` and `read-the-game-list.md`, decision
0022. Findings S1–S4 need an answer from the coordinator before Design; the others can be settled
in Discover or Design.

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| A: Who comes to a game | On the phone, with the chat open, tapping in the answers of several days and looking for the number and for whom to ask | serves – note | The core is right: "no answer" first, two taps, one tap for each further player. S1, S5, S6, S7, S8 |
| B: Warnings in time | A look at the start page: must I act for the coming game? | serves – note | The right warnings at the right moment, and the driver warning earlier than asked helps. But the item does not tell "not answered yet" from "really short". S1, S2, S9, S10 |
| C: After the game | Closing the record of a game that was played | serves – note | Works for a game that was played and kept up during the week. It has no way out for a game that was not played, and costs a tap for each player for a game not kept up. S3, S4, S11, S12 |
| D: The totals on "Attendance" | Seeing the effort of each player, as the coach | serves – note | S4, S13, S14 |

No story conflicts with the vision: the coordinator taps everything in, nothing is sent, and nothing
stored changes by itself. S15 lists rules of existing features that must grow with this change.
S16 is about words.

### Findings
**Answer before Design**

- **S1. Answers stay when a game moves, so the first goal can fail silently.** Story A: "A game moves to another date: the answers stay." Players answered for Saturday; after a move to the next Wednesday, "7 coming" still counts them, so there is no warning, which is the mistake this change exists to prevent. Keeping the answers is right (nothing stored changes by itself), but neither the page, the open item nor the "moved" line in the list of changes says that the answers were given for the old date. A note on the page ("Answers from before the move to Wed 4 Nov") or a line in the change would be enough. Checked: trace of story A against `list-of-changes.md` (kinds, "Duties") and `duties.md` ("Games that change keep their duties"): only duties are named there.
  - 2026-10-31: Fixed: the page says "Answers from before the move" until the change is seen. (coordinator)
- **S2. "6 of 7" does not tell the two situations apart.** With 9 players, the item appears every week from Wednesday until the 7th answer comes, also at "0 of 7" when the coordinator has not announced yet (journey 4 says only "before the weekend"). So in a normal week it says "keep asking", and in a bad week (5 coming, 3 not coming, 1 no answer: 7 cannot be reached) it says the same words. A warning that shows every week gets read like noise, and it is the first goal. The rule itself is the coordinator's ("no answer does not count as coming"); the question is only whether the item may show the difference, for example "Game Sat 31 Oct: 6 of 7 players, 2 no answer". Checked: trace of story B with an example team; `docs/backlog.md` and `idea.md` give no number of players, so how often this happens is not checked.
  - 2026-10-31: Fixed: the item also shows "N no answer". (coordinator)
- **S3. A game that was not played has no way out.** A training can be cancelled; a game cannot. Two cases: a game that left the game list before its date but has answers (it is cancelled), and a game that stays in the list and is not played. By the text of story C each is "not checked" for ever, unless the coordinator taps "Checked". That tap makes each "no answer" absent and, by story D, counts the game in the totals, so players get absences for a game nobody played. Only the case "left the game list and no answers" is covered. At least: a game that left before its date is no open item and does not count. Whether a game in the list needs something like "Not played" is the coordinator's choice. Checked: trace of story C and D against `trainings-and-attendance.md` ("Cancel training") and `duties.md` ("A game whose date has passed was played").
  - 2026-10-31: Fixed: story E, "Not played", which also stops the duties; a game that left the list needs the tap. (coordinator)
- **S4. A checked game can hold a "no answer".** Story A lets a tap on the chosen state go back to "no answer", and story C keeps changes open after "Checked". A role that starts earlier ("Add player" from 1 September, "Correct") also puts a player on the list of a checked game, as "no answer". Story D then says the game counts for that player, but "of 9 games: 8 there · 1 absent" has no place for it. The spec must say how it counts and how the coordinator finds out: at a training the page says "This player now counts as there at 4 earlier trainings" (`players-and-team.md`); here nothing is said. Checked: trace of A, C and D against `players-and-team.md`.
  - 2026-10-31: Fixed: story C. (coordinator)

**Settle in Discover or Design**

- **S5. The list pushes the places down.** The idea asked how the page stays short on a phone; the spec only says that it gets longer, and "Open questions" says none. On a home game of the team, walking through "Not entered" (journey 3) and correcting duties after the game now mean scrolling past 10 to 12 rows and "Extra players". Perhaps fine for the coming game, less so for a past game or one that is checked: a closed list with the count on top would do. Checked: trace of "Open items" 3 and "Season" (`duties.md`, `season.md`).
  - 2026-10-31: Fixed: a checked game shows only the count.
- **S6. "Too few players: 0 of 7" on every coming game, and the count on every row of "Season".** The line shows on all coming games, also weeks ahead, when nobody has been asked. On "Season" each coming game of the whole season gets "9 no answer", which adds to rows that are already full, and says nothing. Proposal: on "Season" only with at least one answer, and the line "Too few" only from the 3 days, or once there is an answer. The row form for 0 coming is not given either: the "part with 0" rule is written for the page only. Checked: trace against `season.md` ("A game", "Labels").
  - 2026-10-31: Fixed: story A.
- **S7. Taps and rows.** "Rows do not move" is good for the thumb, but the reasons below a row push the next rows down, right after a tap, while the coordinator goes on to the next name. The spec does not say when the reasons close; they should close at the tap on another row. A row tapped under "No answer" stays there with "Coming ✓" until the page opens again; that is fine, but only the count is right in the meantime, so the count must stay in view or be read as the truth. Checked: trace of the criteria of story A.
  - 2026-10-31: Fixed: the reasons close at a tap on another row.
- **S8. A reason cannot be taken away.** "One tap adds or changes the reason" has no way back to no reason, other than "Not coming" twice. A tap on the chosen reason could clear it, as a tap on the chosen state does. The spec also does not say whether the reason stays when "Not coming" goes to "Coming" and back. Checked: trace.
  - 2026-10-31: Fixed: a tap on the chosen reason removes it.
- **S9. The warning stays after the game.** "Until the end of that date" and the list of what ends an item leave "6 of 7 players" on the start page on Saturday evening, also after "Checked". A checked game, or simply a game whose start time has passed, should not warn. Checked: trace of story B and C.
  - 2026-10-31: Fixed: the warning ends at the start time or at "Checked".
- **S10. Order of the open items and a small fact.** The first goal comes after "schedule not entered" and "games with open places", below at most 5 rows. A look of a few seconds still sees it, so this is only a note; the coordinator may want the two warnings higher. Also, "those places are at other games than the games of the team" is not always so: `duties.md` says a home game of the club has its places "of any club team, also of the team", and the clash rule names the game itself. The choice not to warn can stay, but the reason is not exact. Checked: `start-page.md`, `duties.md`.
  - 2026-10-31: Fixed: the warnings are high in the order; the reason is rewritten.
- **S11. No answers at all, or an older game: 9 taps.** The story says "one tap", but that holds only when the list was kept up. A game that was not kept up needs a tap for each player, while at a training everyone is "there" at first, and "Checked" on it makes everyone absent. The same holds for filling in the games from 1 September until the release, which the spec allows but makes costly. Without them the totals of story D stay partial for this season. Does the coordinator want those games in the tool? If so, an "All there" is worth asking about. Checked: trace; the number of earlier games is not counted.
  - 2026-10-31: Fixed: no earlier games; the totals start at the release, no "All there". (coordinator)
- **S12. On the day of the game, and after it, the words mix.** Story C changes the buttons and groups from the date of the game on, but the count and "Too few" keep "coming" (story A: "all coming games", and today is coming in `season.md`). So on Saturday morning a list with "There" and "Absent" sits under "6 coming · 1 not coming", before the game has been played. Say which words the count uses, and whether "From the date" should be from the date or from after the start time. "Checked" is also offered on the morning of the game, as at a training, and then makes each "no answer" absent before the game. A past row on "Season" has no "· Checked", where a training has it (`season.md`); the coordinator cannot see there which games are checked. Checked: trace against `season.md`.
  - 2026-10-31: Fixed: the words change after the date; "· Checked" on "Season".
- **S13. Two meanings of "no reason".** The tool starts with a reason called "No reason" (not valid), from `migrations/0005_players_and_settings.sql`, offered below "Not coming". Story D adds "1 absent, no reason" for an absence without a reason. So "absent, not valid" can hold the reason "No reason", next to a separate "absent, no reason". The coordinator will not know which tap made which. Either a tap-free absence counts as that reason, or the words must differ. Checked: search `No reason` in `migrations` and `crates/core/src`.
  - 2026-10-31: Fixed: no reason counts as the reason "No reason".
- **S14. Texts on "Attendance" that assume trainings.** "No trainings yet this season." and "No player had a role at these trainings." block the row of story D's last criterion when a season has games and no trainings, and a tap on a player says "No missed trainings." The list "Left the team" must count games too. Checked: `trainings-and-attendance.md`.
  - 2026-10-31: Fixed: story D.
- **S15. A gone game must stay reachable, and other rules that must grow.** Story C offers "Checked" on a game that left the list and has answers, but `schedule.md` hides a gone game after "seen" or when its date has passed, and `kept_games` keeps a game only for its duties, so the page may not be reachable and the open item may lead to "This game cannot be found." "Delete player" is refused for absences and duties (`players-and-team.md`); the spec does not say what happens to a player with game answers. A driver with a stored "not coming" who is no longer on the list of that date (story A: "does not count") is not settled by story B. The seasons of the item "not checked" are not named; it should follow the current season plus July and August, as the trainings. Checked: `crates/core/src/duties.rs` `kept_games`, `schedule.md`, `players-and-team.md`, decision 0022.
  - 2026-10-31: Fixed: stories A, B and C.
- **S16. Words.** "Extra player" is used throughout and in journey 7, but is not in `docs/language.md`, and it collides with "player": a person the coordinator keeps in the tool. Only the Vision stage adds words. Journey 7 and `change.md` say the driver warning comes "from 3 days before the game"; the spec makes it "at once", which is better for the user, but the vision text should follow at Close. Checked: search of the word list; read of both texts.
  - 2026-10-31: Fixed: words added, journey 7 corrected.

### Second review
Fresh review, 2026-10-31, of the spec as it stands after the fixes above, including story E, which
the first review did not see. Read: this spec, `idea.md`, `docs/vision.md`, `docs/language.md`, all
of `docs/product/` except `login.md`, `choose-club-and-team.md` and `read-the-game-list.md`. Code and
decision 0023 only for the points named below. Numbers go on from S16. S17–S21 need an answer from
the coordinator before Design; the others can be settled in Discover or Design.

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| A: Who comes to a game | On the phone, going back and forth between the chat and the tool, tapping in answers | serves – note | The first goal is well served: the list, the count and the two taps fit the work. S20, S22, S23, S26 |
| B: Warnings in time | A look at the start page: must I act for the coming game? | serves – note | Right warnings, and "N no answer" tells "keep asking" from "really short". S22, S24 |
| C: After the game | Closing the record of a game that was played | serves – note | One tap works, but what the tap settles is a heavy judgement, and a reason cannot be added afterwards. S19, S20, S21, S25, S27 |
| D: The totals on "Attendance" | Seeing the effort of each player, as the coach | serves – note | Fits journey 2, but it inherits the absences of S21. |
| E: A game that was not played | Making sure nobody gets a duty or an absence for a game that did not happen | does not serve, as written | It works for a game that stays in the game list and is not played. It fails in the two other likely cases: a game cancelled before its date, and a game played later. S17, S18, S19, S28 |

No story conflicts with the vision: nothing is sent, and nothing stored changes by itself.

#### Findings
**Answer before Design**

- **S17. "Not played" stays on a game that moves to a later date.** A game that is not played is often played later. If the game list then moves the same game (`duties.md`: "A moved game keeps its places"), the coordinator has already tapped "Not played". Story A says what happens to answers at a move, but nothing says what happens to "Not played" or to "Checked". Read literally, the game is coming again while its duties and attendance do not count. Then the warnings, which also leave out a game that is not counted, could stay silent on a game that will be played, and after it the drivers get no credit. The word list also says "a past game". The spec must say whether a move to a later date ends "Not played", or at least shows it. A checked game that moves has the same question: a checked game gives no warning, and its "no answer" players were settled as absent. How often the game list moves a game that was not played is not checked. Checked: trace of story E against story A and `duties.md`.
  - 2026-10-31: Fixed: the marks hold only for the date tapped. (coordinator)
- **S18. A game cancelled before its date starts to count by itself, and "Not played" cannot be tapped in time.** Story E's aim is that nobody gets a duty for such a game. But a coming game that left the game list does not count only while it is coming. Once its date passes, it counts: in `crates/core/src/duties.rs`, `game_places` has `counts = past || (in_list && required)`, and the test `a_coming_game_that_is_gone_does_not_count_and_a_past_one_does` holds that. So the drivers are "done" from the next day. The coordinator sees the change on Thursday, opens the game, and finds no button, because "a coming game has no 'Not played'". The change is then marked as seen, and from Sunday nothing points to the game: story C says it is no open item. The rule that a reading changes nothing can stay. The gap is the moment of the tap. Either "Not played" must be possible on a coming game that left the game list, or a past game that left before its date, with duties or answers, needs its own open item. Checked: code as named, and trace of stories A, C and E against `list-of-changes.md`.
  - 2026-10-31: Fixed: "Not played" also on coming games. (coordinator)
- **S19. A game of the team with no answers and no duties has no way out.** "Not played" is offered only on a past game "with duties or attendance". A game of the team that stays in the game list, was not played, has no answers and no player on a place gets the open item "Game … not checked". The only way to clear it is "Checked", which makes everyone absent: the problem of S3 again. It is a likely case, because a coordinator who hears early that a game is off taps nothing. "Not played" should be on every past game of the team, whatever is stored. The spec must also say what "with duties" means for a home game of another club team: a place that names any team, or only a place of the team. Checked: trace of stories C and E.
  - 2026-10-31: Fixed: on each game of the team. (coordinator)
- **S20. A reason cannot be added later.** The reasons show only "after 'Not coming'", and close at a tap on another row. No criterion opens them again for a player who already has the state. In the chat a reason often comes later ("Eva can't come", the next day "she's ill"). Before the game, a tap on "Not coming" goes back to "no answer", and a second tap brings the reasons back, so 2 taps through a wrong state. On a checked game it is worse: "a tap on the chosen state does nothing", so the coordinator must tap "There" and then "Absent". That is exactly the case of a player settled by "Checked" whose reason the coordinator then learns. At a training a tap on an absent player opens the reasons (`trainings-and-attendance.md`), so a tap on the row, or on the name, would match. Checked: trace of stories A and C.
  - 2026-10-31: Fixed: the link "Reason". (coordinator)
- **S21. "Checked" turns "no answer" into an absence that counts as not valid, and nobody decided that.** `idea.md` left this open ("must each become 'there' or 'absent'"), but the spec gives the choice with no source, where other choices name the coordinator. Without a reason, an absence counts as "No reason", which is not valid (`migrations/0005_players_and_settings.sql`). So a player whom nobody asked, or who came without answering, gets a not valid absence in the numbers the coach reads for effort, and the vision says the reason shows how invested a player is (change log, 2026-10-14). The message "2 players with no answer are absent." comes after the tap. Undo exists, but only for the whole game. It may be the right choice, since the list has "No answer" at the top; the coordinator should make it. Options: name the players before the tap, or keep it as it is. How many players stay "no answer" in a real week is not checked. Checked: trace of stories C and D; search `No reason` in `migrations`.
  - 2026-10-31: Kept: "No reason", not valid; the reason can be corrected. (coordinator)

**Settle in Discover or Design**

- **S22. "One rule" is not what the spec describes.** Story B says "the page of the game and the open item give the same answer". But the page shows "Too few players" before the 3 days once there is an answer, and on all of the day of the game, also after the start time. The item shows only from 3 days until the start time. On a checked game of today the page's criterion says nothing about stopping, while the item stops. The difference before the 3 days is good for the user: the words should say one count, with two different times. Also say whether "an answer" includes extra players, as it does on "Season". Checked: trace of story A against story B.
  - 2026-10-31: Fixed: one rule; the page and the item have the same end.
- **S23. The note "Answers from before the move" is weak protection for the first goal.** It shows only on the page of the game, until the change is seen, and "Mark all as seen" is one tap on another page. After that "7 coming" stands with no hint, and the row in "Next" never shows it. A line in the change itself, next to "has duties", was the other half of what S1 proposed. Also, a game that moves from the past with answers but no duties has no change at all (`list-of-changes.md` keeps a past change only for duties), so the note is not defined for it. Checked: trace against `list-of-changes.md`.
  - 2026-10-31: Kept: a note; the count gives the warning.
- **S24. The driver warning has no end at the start time.** S9 fixed this for "too few" only. The driver item is for "a coming game", and today counts as coming (`season.md`), so "driver Anna is not coming" stays on Saturday evening after the game, until "Checked" or midnight. Checked: trace of story B.
  - 2026-10-31: Fixed: the driver warning ends at the start time.
- **S25. A driver who was absent still counts as having driven.** After the game "Anna · not coming" is no longer a warning, and the place counts as done unless corrected (`duties.md`). The page has both facts and does not connect them, so "Absent" and "1 drive done" stand side by side for the same game, against core value 2. A note at a past place ("Anna · absent") would be enough; nothing should change by itself. The same holds for table and referee places at that game. Checked: trace of story C against `duties.md`.
  - 2026-10-31: Fixed: "Anna · absent"; the correction stays with the coordinator.
- **S26. Coming back from the chat.** The real work goes back and forth between the chat and the tool, and at a training the page reloads the stored marks when the phone comes back. Story A says that the groups sort again "when the page is opened". It does not say whether coming back counts. If it does, the rows jump after each look at the chat, and a thumb on its way to a row taps the wrong player. If not, the groups stay stale, which is harmless. Checked: trace against `trainings-and-attendance.md`.
  - 2026-10-31: Fixed: rows stay while the page is open.
- **S27. "· Checked" in "Last".** Story C puts it on "Season" only, story E puts "· Not played" on "Season" and in "Last", and "Last" shows it for a training. So the row in "Last" does not say if the game is done. "Interface" names "Next" and "Season" here, and "Next" has no past games. `start-page.md` also shows in "Last" only games with places of the team, so a game without places cannot be reached from there. Checked: `start-page.md`.
  - 2026-10-31: Fixed: also in "Last".
- **S28. The rest of a not played game.** The spec does not say what the page shows: can answers and extra players still be changed, and do the count and the list stay? A coming game known to be off that stays in the game list keeps warning until its start time, with no way to stop it. That is a small nuisance, and it needs no change if the coordinator accepts it. Checked: trace of story E.
  - 2026-10-31: Fixed: story E.
- **S29. Small cases.** After "Checked" every player on the list has a stored state, so a player added by mistake can no longer be deleted ("Delete player" is refused), only hidden. This differs from a training, where "there" is not stored. Story B covers a game that is no longer a game of the team, but nothing says what happens to its answers, whether the page stays reachable, or whether it counts in the totals, for example after another team is chosen. Checked: trace against `players-and-team.md`.
  - 2026-10-31: Fixed: a player with answers can be hidden; the other small cases stay as written.

### Third review
Fresh review, 2026-10-31, of the spec as it stands after the fixes of the second review. Read: this
spec, `idea.md`, `docs/vision.md`, `docs/language.md`, all of `docs/product/` except `login.md` and
`choose-club-and-team.md`. Code only for the points named below. Numbers go on from S29. S30 and S31
need an answer from the coordinator before Design; the others can be settled in Discover or Design.

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| A: Who comes to a game | On the phone, going back and forth between the chat and the tool, tapping in answers and reading the number | serves – note | Two taps, the groups and the count fit the work. S30, S32, S33 |
| B: Warnings in time | A look at the start page: must I act for the coming game? | serves – note | Right warnings, high in the order. It stays silent on a moved game with old answers. S30, S31 |
| C: After the game | Closing the record of a game that was played | serves – note | One tap works after a game. The tap is also offered before the game, and what it settled has no defined way back in two cases. S31, S34, S35 |
| D: The totals on "Attendance" | Seeing the effort of each player, as the coach | serves – note | S36, S39 |
| E: A game that was not played | Making sure nobody gets a duty or an absence for a game that did not happen | serves – note | The three gaps of the second review are closed. One rule of duties is still missing, and one case still depends on the coordinator remembering. S37, S38 |

No story conflicts with the vision: nothing is sent, and nothing stored changes by itself. The spec
is now five stories, and story E changes released rules of duties, against "small and correct beats
complete". Each part has a reason, so this is no finding, but the first goal needs only stories A
and B, and the coordinator needs it each week of the season. Whether it can go out first is a
question for Design.

#### Findings
**Answer before Design**

- **S30. A moved game with old answers still gets no warning, and "the count gives the warning" is not true in that case.** S23 was kept with that reason. But S1 is exactly the case where the count gives none: 7 players answered for Saturday, the game moves, and "7 coming" stands, so no line and no open item. Only the page note is left, and it ends at "seen", while "Mark all as seen" is one tap on another page that never shows the note. Story E makes this case more likely than when S1 was written: a game marked "Not played" and moved later is "a normal game of its new date again, with its answers", weeks later, when those answers are worth least. The answers can stay. The question is only how long the note lives: for example until the game has started, or until a tap on the page itself ("Answers still right"), or until each answer is tapped again. The same line in "Next" would make it visible without opening the game. A correction to S23: a game that moves from a past date to a coming date does get a change, also without duties; only a move from a past date to a past date gets none, and then no warning is at stake. How often a game moves after answers are in is not checked. Checked: trace of stories A, B and E against `list-of-changes.md`; `change_between` in `crates/core/src/comparison.rs`.
  - 2026-10-31: Fixed: "Answers checked" and an open item. (coordinator)
- **S31. "Checked" before the game ends the warnings and makes "no answer" absent.** S12 raised this and the fix covered only the words. A game of today has "Checked" from midnight. On Saturday morning, with 5 coming and 3 no answer, one tap settles those 3 as "not coming", without a reason, and the "too few" and driver warnings are gone ("the game is checked"), on the day the first goal matters most. At a training this is harmless: "Checked" adds nothing there. Here it changes the record and silences the tool. Offering "Checked" from the start time (for a game without a known start time, from the date, as all such rules) would remove the case. The coordinator may want it from the date, for a game with a wrong time; the choice is theirs. Linked: on the evening of the game, the record is closed with the words "coming" and "not coming", and a checked game of today keeps showing "8 coming · 1 not coming", while its message says "absent". The same moment, the start time or "Checked", could change the words. Checked: trace of stories A, B and C against `trainings-and-attendance.md` ("Checked") and S12.
  - 2026-10-31: Fixed: "Checked" from the start time. (coordinator)

**Settle in Discover or Design**

- **S32. The reasons open at each "Not coming", also when there is no reason to give.** The spec's own fact is that the chat often gives none. So in the common case a panel of up to 6 reasons opens below the row, pushes the next rows down under the thumb, and jumps back at the next tap. That goes against the aim of "Rows do not move", and the tap must also both close the panel and save on the other row, which the spec does not say. Now that the link "Reason" exists (S20), the panel no longer has to open by itself; a short line of the reasons in the row's own height, or a fixed place for the panel, would also do. On the other side, the vision wants the reason, because it shows how invested a player is, and a panel that does not open gets fewer reasons. That is the trade-off to decide. Checked: trace of story A; not tried on a phone.
  - 2026-10-31: Fixed: reasons only through the link.
- **S33. The count can scroll out of view.** S7 said the count must stay in view, because the rows do not move and it is the only live truth; the fix covered only the closing of the reasons. With 10 to 12 rows of two buttons, the top of the page is gone at the lower rows, and the number is the point of the page. A count that stays at the top of the screen is enough. Checked: trace of story A; the height of a row is not checked.
  - 2026-10-31: Fixed: the count stays in view.
- **S34. The way back of a settled player has holes.** "Undo check" brings "those players" back to "no answer", and "players marked by hand keep their state". Not said: (a) a settled player who then gets a reason through the link "Reason", the very case of S20: is that marked by hand? The reason should survive the undo, so yes. (b) A game with "Checked" that moves: story E makes it a normal game with its answers, and read literally the settled players stay "not coming", so the count shows answers nobody gave, and "N no answer" is too low. They should go back to "no answer", as at "Undo check". (c) A player back at "no answer" with a kept reason (story A keeps the reason only "to 'Coming' and back"): does "Checked" give "absent, without a reason", or the kept reason? Checked: trace of stories A, C and E.
  - 2026-10-31: Fixed: "Undo check".
- **S35. A past game that was not kept up, also each game from before the release, opens with the full list.** S5 closed the list for a checked game only. A past game before the release is never checked, so it opens with 9 to 12 rows "No answer" above the places, and "Checked" on it makes everyone absent. These are the games of this season since 1 September, and correcting a duty there is journey "After the game" of `duties.md`. A game from before the release could show the list closed, as a checked game. Checked: trace of story C against the rule "games before the release are not entered"; the number of games is not counted.
  - 2026-10-31: Fixed: the list is closed.
- **S36. "No reason" is an ordinary reason.** Spec: an absence without a reason "counts under the existing reason 'No reason'". In the tool that reason is one row among the reasons, with no mark that sets it apart, and it can be renamed, changed to valid and hidden (`weekly-trainings-and-reasons.md`). So the coordinator's choice in S21, "not valid", holds only while nobody changes that row, and the tool has no safe way to find it by name. Design must tie the rule to something fixed, or an absence without a reason must count as not valid on its own and show "No reason" as a text. The second also keeps a game absence from looking different after a rename. Checked: `migrations/0005_players_and_settings.sql`; search `No reason` in `crates/` and `web/src`: no use in code.
  - 2026-10-31: Fixed: no reason counts as not valid; replaces the answer to S13.
- **S37. "Not played" says nothing about clashes.** Story E names totals, warnings, open items, messages and overviews. The warning to Design names "the choice of a player", but no criterion does. In `clashes` (`crates/core/src/duties.rs`) each game of the team and each duty of the season can block a choice or give a note. Read literally, a game that is off still blocks a player for a table place at that time ("at another game at almost the same time"), which is just the moment such a place is free to fill. A not played game should give no clash. Likewise the label "Open places" on "Season" should follow the open item. Checked: code as named; trace of story E against `duties.md` ("Clashes") and `season.md`.
  - 2026-10-31: Fixed: story E.
- **S38. A game that left the game list still counts by itself once its date has passed.** S18 was fixed with the button on coming games, which is right. But story E's own text names this problem, and it stays when the coordinator does not tap: the change is marked as seen, "Mark all as seen" does that without opening the game, and the next day the drivers have "1 done", with no open item (story C excludes it). That is against core value 2, and nobody will notice. Without changing the rule: the line of the change could say what to do ("has duties · was it not played? Open the game"). Also say where a gone game with answers but no duties is reached from after "seen": `schedule.md` hides it, and `list-of-changes.md` gives it no link. Checked: trace of story E against `list-of-changes.md` and `schedule.md`.
  - 2026-10-31: Kept: facts of 2026-10-27; the list of changes.
- **S39. Small cases.** (a) "Interface" still says "· Not played" on past games, "the page of a past game: 'Not played'" and "at a past game 'Checked'", and does not name "Last"; stories C and E say each game, coming or past, and today. (b) Story A says a not played game leaves "the week messages", story E "the messages and duty overviews". (c) "8 there" on a past game: with or without extra players? "Coming" includes them, story D never counts them, and a checked game shows only the count, so the extra players are hidden. (d) "Anna · absent" is at the driver place only; S25 also named table and referee places at that game, and the text costs nothing there. (e) The list below "2 games: too few players" has the row form of the other items, without the "5 of 7", which is the fact the coordinator wants. (f) A tap on a player on "Attendance" lists games and trainings with "the date and the reason": nothing tells them apart. (g) "The games start at the release" makes sense only for that season. (h) `change.md` still reads as if the driver warning starts 3 days before the game (S16). (i) A game page says "the list" next to "the game list" and "the list of changes"; the word list has no word for it, so the words of the page should avoid the bare "list". Checked: read of this spec against `start-page.md`, `trainings-and-attendance.md` and `change.md`.
  - 2026-10-31: Fixed: "Interface".
