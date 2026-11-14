# Idea: Review of the page "Season"

**Status**: ready for discovery
**Purpose fit**: supporting work for both core values: "Season" is where the coordinator finds and corrects the record (effort and duties), and plans the trainings. "Where we are going": each page shows only what that work needs.
**Date**: 2026-11-14

## The problem
"Season" was made on 2026-10-25 with three jobs: find any event, plan the trainings ("Add
training", "Cancel" on a training day), and be the work list (the labels "Not entered" and "Open
places"). Since then the start page took over the work list: its open items list the home games
without a committee schedule, the open places, and every past event that is not checked, each with
its own list of games.

The layout is still made for three jobs. On a phone the first event starts low (about 660 of 844
pixels, ideas list 2026-10-25), below the season choice, the "Game list" card, "Add training",
three rows of labels (8 buttons; labels of one row add, labels of different rows narrow), and
"Show cancelled". Its rows are cards with a "Team" badge and two full team names, while the start
page has flat rows ("Home game against …"), an orange bar for a game of the team, a blue bar for
the other games with places of the team, and lighter rows for events with nothing left to do (ideas
list 2026-11-10).

## The idea
A proposal from the conversation of 2026-11-14, for Discover to test, not a decision:
- **Purpose:** "Season" shows the whole season, to find, plan and correct events. The start page
  is the list of things to do.
- **The rows of the start page**, with the lighter rows; the week headings stay (a season has
  empty weeks).
- **One row of filters, one on at a time:** "All · Team games · Duty games · Trainings · Schedule
  not entered". "Entered" and "Open places" go: the open items do that work. "Schedule not entered"
  stays because the committee's schedule covers a whole half season and the coordinator enters it
  in one go; it follows the rule of the change "Schedule not entered for each half season".
- **Opens at today**, with "Earlier weeks" for the past, in place of the labels "Coming" and
  "Past".
- **No "Show cancelled"**: a cancelled training shows lighter.
- **Smaller controls:** "Add training" and the season choice next to the title.
- **The "Game list" card moves to "More"** (the coordinator seldom reads the game list by hand);
  the open items of the game list link there.

## Why now
The start page changed (v0.11.0 to v0.15.0), so the two pages no longer look alike or share the
work in a clear way. The coordinator no longer uses the sheet, so "Season" is the only place to
find the record.

## How it fits
- Changes [Season](../../product/season.md), and the links of the open items of the game list on
  the [start page](../../product/start-page.md) and [read the game list](../../product/read-the-game-list.md).
- After the change [Schedule not entered for each half season](../2026-11-14-schedule-per-half-season/idea.md).
- Takes in the ideas list lines of 2026-10-25 (the first event is low), 2026-10-27 (the phrase
  "possible duty games"; past home games keep "Schedule not entered" for ever on "Season") and
  2026-11-10 (the look of the start page).

## User view
A few times a season the coordinator opens "Season" to look ahead (a game in three weeks), to
correct a past event, to add or cancel a training, or, about twice a season, to walk through the
home games with "Schedule not entered". The page opens at today with the events at once.

## Open questions
- A row of 5 filters on a phone of 320 pixels: does it fit, and is one at a time right (team games
  and trainings together)?
- What does "Duty games" show: only the games with places of the team (the word "duty game"), or
  every home game of another club team? Where do the home games of other club teams without places
  of the team show, and is "possible duty games" still needed as a word?
- "Earlier weeks": all of the past at once, or a few weeks for each tap? Does a return from a past
  event keep the place?
- Past home games show "Schedule not entered" for ever (ideas list 2026-10-27): still on the row?
- Where exactly in "More" goes the game list, and what happens to its result line and warnings?
- The page "Attendance" also lists trainings: overlap with "Season" (ideas list 2026-10-18 and
  2026-11-09)?
