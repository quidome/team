# 0026 – The committee message always has a text

**Status**: valid
**Date**: 2026-10-31
**Change**: [2026-10-18-duties](../changes/2026-10-18-duties/) (story [the committee message](../changes/done/2026-10-31-committee-message/))

## Context
Decision 0025 says that a past place that stayed open alone gives no message of the week and no
button, because it announces nothing to the team. The referee committee reads the same week for
another reason: which player did or does each place of the team.

## Decision
- **With a chosen team, the committee message always has a text**, so its button shows in every week. A week without places says "Geen tafel- of scheidsrechtersdiensten deze week."
- **A past place that stayed open shows "niemand", also alone.** "Geen …" would be false: the team had a place.
- **Full names come from `team_core::full_names`, and the club and team are joined in the core**, with the lines of `duty_lines` (decisions 0024, 0025).
- **All pages of text to copy use one component** (`MessagePage.svelte`), and the routes use one function (`message_view`).

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Always a text (chosen) | The coordinator can tell the committee "nothing this week" | The part "Messages" always shows with a team |
| A button only in a week with places | A shorter start page | Nothing to send when the committee asks |
| A past open place alone gives "Geen …" | The same rule as the message of the week | False for the committee |

## Consequences
- The two messages of one week can differ: the committee message can exist when the message of the week does not.
- Two players with the same full name are not told apart.
