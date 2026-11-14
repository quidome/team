# Idea: Schedule not entered for each half season

**Status**: ready for discovery
**Purpose fit**: core value 2 (share the duties fairly and easily): the open item asks for the committee schedule when the coordinator can enter it, and is silent when nothing can be done yet. Fewer wrong reminders also serve "Where we are going" (attention only when there is something to do).
**Date**: 2026-11-14

## The problem
The referee committee makes its schedule of table and referee places once for each half season,
for the home games of all club teams. It can do that only when the club teams have their
competitions for that half. The coordinator enters that schedule in one go, to make it complete.

The tool does not know this. Its open item "schedule not entered" looks 27 days ahead, game by
game. So:
- now only MSE 1 (a senior team, one competition for the whole season) has games after
  1 January. From about 5 December its January home games give the reminder "schedule not
  entered", while the committee cannot make a schedule from one team;
- when the schedule of the second half comes, the home games more than 27 days ahead are not in
  the open item, so it is not the list for the work of entering it.

The game list cannot say when the schedules are complete: with games of 5 teams the tool cannot
tell whether a sixth team comes later or does not take part (one club team withdrew from the
competition this season).

## The idea
- **The test is per half season, not per game or weekend.** A half season counts when more than
  2 club teams have their competition for it in the game list (the coordinator's threshold:
  "games for more than 2 teams means we must start").
- **Until then** no home game of that half is an open item "schedule not entered", also not
  within 27 days.
- **From then** every home game of that half without the committee schedule counts, not only the
  next 27 days. The item goes away by itself when all known home games of the half are entered,
  and comes back by itself when more teams get their games.
- **The filter "Schedule not entered" on "Season"** follows the same rule, so the item and the
  filter show the same games.
- **The coordinator sees how far the schedules reach**, for example "Second half: 1 of 5 youth
  teams known", with for each club team its home games in each half and its last known game. The
  tool shows; the coordinator decides when it is enough.

Facts from the game list (a copy of 2026-10-11, club 37, in the test data of the reader):
- youth teams play a first and a second half competition: the competition name has "1e Helft" or
  "2e Helft", often with another division in the second half. In 2025-2026: 14 Sept – 19 Dec and
  17 Jan – 2 May;
- senior teams (MSE, VSE) play one competition for the whole season (2025-2026: 14 Sept – 2 May);
- in 2026-2027 the five youth teams have only a first half (until 12 December); MSE 1 plays until
  10 April 2027.

So the club teams expected in the second half are those with a first-half competition. A home
game of a senior team belongs to the half season of its date (1 January splits them).

## Why now
From about 5 December 2026 the 27-day window reaches the January home games of MSE 1, and the
wrong reminders start. The first half ends on 12 December; the second-half schedules follow.

## How it fits
- Changes the open item 4 of the [start page](../../product/start-page.md) and the label "Not
  entered" of [Season](../../product/season.md); the places themselves ([duties](../../product/duties.md)) do not change.
- The open item "open places" does not change: a place gets a player only after the schedule is
  entered.
- Comes before the [review of "Season"](../2026-11-14-review-of-season/idea.md), which keeps the
  filter "Schedule not entered" and uses this rule.
- Words: season, half season, competition (added to `docs/language.md` on 2026-11-14).

## User view
In December the coordinator sees no reminder for the January games of MSE 1, and a line that only
1 of 5 youth teams is known for the second half. When the youth teams get their second half, the
count rises; with 3 teams the open item shows all home games of the second half without a
schedule. When the committee's schedule comes, the coordinator walks through them on "Season"
with the filter "Schedule not entered".

## Open questions
- What does the item ask: "ask the committee", or "enter the committee's schedule"? Its text.
- "More than 2 club teams": all club teams, or only those with a half competition? A senior team
  always has the second half, so it would count before any youth team is known.
- Where does the count for each club team show: on "Season", in the open item, or both?
- The words "1e Helft" / "2e Helft" are no promise of the game list (seen in two seasons of one
  club). What when a competition name has neither: one competition for the season?
- The club team that withdrew: does it have a competition in the game list? Its home games ask
  for places today (ideas list, 2026-11-01). Does it count in "of 5"?
- A home game of a half that does not count yet, very near (the committee schedules one team on
  its own): the tool stays silent. Acceptable?
- The open items in July and August also cover the season before: does that change anything here?
