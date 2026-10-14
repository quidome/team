# Idea: Attendance at training

**Status**: ready for discovery
**Vision fit**: priority 2 of the vision and user journey 2; core value 1 ("attendance at training takes seconds, on a phone, with one hand"). It also brings the players into the tool, which duties, messages and duty totals (priorities 3 to 5) need.
**Date**: 2026-10-14

## The problem
The coordinator is also the coach of the team. Attendance tells the coach how much effort and
energy each player gives to the team. Today the coordinator marks attendance in a Google Sheet,
at each training, for every player who trains. That is more than the players who play games.

An absence is not always the same: the reason shows how invested a player is in the team.
School is a valid reason to miss a training; other reasons are less valid.

(Coordinator, 2026-10-14.)

## The idea
- **Players.** The coordinator types in the players of the team: first name and last name, and whether the player plays games or only trains. The coordinator can change and remove players during the season. In the screens a player is shown by name only. (The coordinator's sheet has 14 players: 10 play games, 4 only train.)
- **Trainings.** The team has a fixed schedule: every Tuesday from the first Tuesday of September to the last Tuesday of May. Some trainings are cancelled (holidays); now and then an extra training is planned (rare). A cancelled training does not count for anyone.
- **Attendance.** For a training, all players (also those who only train) start as "there". The coordinator taps who is missing and gives a reason. The coordinator can also note an absence before the training, for example when a player said so in the chat.
- **Reasons.** A preset list (for example school, ill, injured, holiday, work, no reason). Each reason is valid or not valid. The coordinator can add more reasons in the app.
- **Season totals.** For each player, for the season: trainings there, absent with a valid reason, absent without a valid reason.

Not in this idea:
- **Attendance at games.** Also important to the coach, but a separate, next change: it needs decisions about games (who was selected, who played, and how it relates to drives and duties).
- **Jersey number, birthday and association ID.** "Handy at some point"; they come with a feature that needs them (for example the message to the referee committee).
- **Reading the players from a sheet.** The coordinator types them in; 14 names, once a season.
- Totals for recent weeks, or trends.

## Why now
The games come in from the game list (v0.2.0, released 2026-10-14). Attendance at training is
the next priority in the vision and the work that is done every week. Players are needed for
everything after it.

## How it fits
- Vision: priority 2 ("attendance at training"), which now names attendance at games as the step after it; user journey 2.
- Words (changed 2026-10-14): **player** now also covers a player who only trains; **attendance** includes the reason; **reason** is a new word.
- Existing features: none use players yet. The schedule and the list of changes are about games and do not change. The season rule (a season starts on 1 July) already exists and can decide which trainings belong to a season.
- One coordinator, one team (vision). The players belong to the team of the current season's choice.

## User view
The coordinator stands at the side of the hall, maybe with a ball in one hand, while the players
walk in. The coordinator opens today's training on the phone: all players show as "there". A tap
on a missing player asks for a reason. It must take seconds. Before the training the coordinator
may already know from the chat that someone cannot come, and notes it then.

At the start of the season the coordinator types in the players once. Afterwards, the coach looks
at the season totals to see who comes regularly and who misses trainings without a valid reason.

Awkward: a player who joins or leaves halfway the season (totals over the trainings since
joining?); a training that is cancelled on the day itself.

## Open questions
For /discover:
1. A player without a last name: the coordinator's sheet has 3 such players. Is the last name required?
2. Does a training need a start time and a place, or only a date? (The schedule is every Tuesday; the sheet's Games tab has locations, but not for trainings.)
3. How does the coordinator set up the season's schedule: the weekday, the first and the last date, then cancel single dates? What about a holiday week of several trainings?
4. What counts in the totals: only trainings in the past? What happens with a training where no one opened the attendance (no marks): everyone there, or "not recorded"?
5. A player who joins or leaves during the season: which trainings count for that player?
6. Can a reason be renamed, removed or changed between valid and not valid later, and what happens to earlier absences with that reason?
7. Where in the app does it live: the start page (today's training when there is one), a separate page, or both?
8. Must the coordinator see the attendance of one past training again, and correct it?
9. A new season: are the players kept (most players stay), and how does the coordinator remove those who left?
