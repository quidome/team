# 0034 – The committee message on the start page only in weeks with places

**Status**: valid
**Date**: 2026-11-11
**Change**: [2026-11-10-quieter-start-page](../changes/done/2026-11-10-quieter-start-page/)

## Context
Decision 0026 gave the committee message a text in every week, so its button showed on the start
page whenever a team was chosen, also in a week without table or referee places of the team. On
2026-11-10 the coordinator found the start page overwhelming: it showed everything that is true,
also buttons with nothing to do this week. The reason of 0026 is that the coordinator can tell
the referee committee "nothing this week" when it asks.

## Decision
- **The start page offers "Committee message ›" only in a week (Monday to Sunday) with table or
  referee places of the team that count, also on a day that has passed:** the same games as the
  lines of its text (`team_core::week_has_duty_places`, which reuses the rule of the text).
- **The page "Duties" always has the button "Committee message"** for the current season, next to
  the two duty overviews, also without a team (the page then says "No message: no team chosen."),
  as the overviews. "Back" there returns to "Duties" (`?from=duties`).
- **The text does not change:** it still exists in every week with a team (decision 0026), with
  "Geen tafel- of scheidsrechtersdiensten deze week." in a week without places.

This replaces the part of decision 0026 that shows the button on the start page in every week.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Start page only with places, always on "Duties" (chosen) | A quieter start page; the weekly action stays one tap away in the weeks that need it; "nothing this week" is still on "Duties" | One tap more when the committee asks in a week without places |
| Only on "Duties" | The shortest start page | One tap more each week with places; journey 5 is a weekly action before the weekend |
| On the start page in every week (0026) | Always the same place | A button with nothing to do in many weeks |

## Consequences
- On the Sunday after a Saturday with places, the link still shows (the week still has them).
- The two links of the start page can differ: the message of the week can show without the
  committee link (a game of the team, no places), and the committee link without the message of
  the week (only past places that stayed open).
- The start page flag and the text agree by construction; a core test checks it on each case.
