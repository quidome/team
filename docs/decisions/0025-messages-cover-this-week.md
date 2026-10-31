# 0025 – Messages cover this week, Monday to Sunday

**Status**: valid; 0026 adds the committee message
**Date**: 2026-10-31
**Change**: [2026-10-18-duties](../changes/done/2026-10-18-duties/) (story [the message of the week](../changes/done/2026-10-30-message-of-the-week/))

## Context
Decision 0024 made the message for the next game: the team's next game, also weeks away, and the
table and referee places from today to the coming Sunday. Before its release the coordinator said
what messages are for (vision of 2026-10-30): they announce what comes in a week, and the team
message is the start of a poll in the chat. A message about one next game does not fit a week with
two games, and "today to Sunday" is empty on a Sunday.

## Decision
- **A week is Monday to Sunday, Dutch time; a message covers the week that holds today**, also on a Saturday or Sunday (`team_core::week_of`). One week only.
- **The message of the week has two parts with different rules.** A game of the team has a block from today to Sunday, because a poll under a played game makes no sense. The table and referee places stay for the whole week, also days that have passed.
- **A past place shows who did it** (`who_did`, decision 0023), and "niemand" for "Nobody" and for a place that stayed open. Such an open place alone gives no message and no button.
- **One way to write open places** in all texts: the names, a comma, "nog N open". "Scheidsrechter" comes before "Tafel".
- **The lines for table and referee are one function, with the names as an input** (`duty_lines`), so the committee message (full names) and the duty overviews use the same rule.
- **The older route, flag and page are replaced with no alias**: nothing of them was released.
- From decision 0024 stay: one rule in the core, made at each request, compiled-in texts, the button from the same function as the page (now `api::messages::week_text`), and "Thuis" by the club's own hall.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| A played game of this week leaves the message (chosen) | The poll is for games to come | Differs from the places, which stay |
| A played game stays, marked "gespeeld" | One rule for the whole week | A longer message, with something the chat does not need |
| A past open place shows "niemand" (chosen) | True, and one rule for all past places | Says nobody did it, also when only the record is missing |
| A past open place shows "nog 1 open" | No new text | Wrong: the game is over |
| A past open place leaves the message | Shorter | The place disappears without a sign |

## Consequences
- On a Sunday evening the next week cannot be announced yet (an idea in `docs/ideas.md`).
- A week around 1 July shows only the games of the current season.
- The test data of the browser tests has no game in the week of the test clock, and other tests depend on that. So the browser tests with a message fix the answers of the server; `crates/server/tests/messages.rs` tests the text from stored data, and that the button and the message agree.
