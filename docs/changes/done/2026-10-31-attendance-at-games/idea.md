# Idea: Attendance at games

**Status**: ready for discovery
**Vision fit**: Priority 7 and journey 7 (changed 2026-10-31). Core value 1 ("capture fast, keep it correct"). It replaces work the coordinator does now in the chat, and the report (priority 8) needs its numbers.
**Date**: 2026-10-31

## The problem
After the coordinator announces the week, players say in the chat that they come or do not come.
The coordinator must see in time when there are too few players, because then extra players must
be found from other teams. Today that overview is only in the chat. A driver who does not come to
the game is also easy to miss.

## The idea
As told by the coordinator (2026-10-31):

- Each game of the team has a list of the players with the role "plays games". Each has one of three states: "coming", "not coming" with a reason, or "no answer".
- The coordinator updates the list during the week: open the tool, change the attendance of the coming game.
- After the game the coordinator confirms the list with "Checked", as at a training.
- Fewer than 7 players "coming" is a problem. It gives an open item from 3 days before the game: from Wednesday for a game on Saturday, from Thursday for a game on Sunday. This holds for home games and away games alike. "No answer" does not count as coming.
- Extra players from other teams are a number on the game ("2 extra players"), which adds to the count. They are not named, and they are not added to the tool as players.
- A player on a driver place who is marked "not coming" is flagged. The tool does not remove the driver: what the coordinator stored never changes by itself.

## Why now
Duties is released (v0.7.0), and attendance at games is the next priority. The coordinator needs it
each week of the season. It comes before the report, which needs its numbers.

## How it fits
- It works as attendance at training where it can: the reasons, a tap that saves at once, "Checked".
- It differs in the third state. At a training everyone is "there" at first; before a game that would hide who has not answered, so a shortage would not show.
- The open items follow decision 0022: each comes from the rule of its source.
- The role "only trains" keeps its meaning: such a player is not on the list. An extra player is counted by the number.

## User view
The coordinator announces the week, reads the answers in the chat, and on the phone opens the page
of the game and taps the answers in. The count shows at once. From Wednesday the start page says
when a Saturday game has too few players, or a driver who does not come.

## Open questions
- What happens with "no answer" at "Checked": must each become "there" or "absent", and what is shown if the coordinator never checks a game? Is an unchecked past game an open item?
- Where does the list live: on the page of the game (assumed), and how does the page of a game stay short on a phone?
- What does the page "Attendance" show: are games in the same totals as trainings, or in their own column? How does a player who joined later count?
- Does the coordinator see the count on "Season" and in "Next" on the start page, and not only on the page of the game?
- Does an extra player count in anything but the count of 7?
- A game that moves or leaves the game list: what happens with its attendance? (What is stored is never changed by a reading.)
- A game with no known start time, and a game before the 3 days: is the count shown without an open item?
- Is 7 a fixed number or a setting? The coordinator named one number for the team.

## Not in scope
- The driver proposal: a separate small change (one driver for every 4 players; `docs/backlog.md`). The tool does not use the answers to change the number of drivers.
- Names of extra players.
- Reading the poll from the chat: the coordinator taps the answers in.
- The report of what each player did (priority 8).
