# 0024 – Messages made in the core, in Dutch, from what is stored

**Status**: valid
**Date**: 2026-10-29
**Change**: [2026-10-18-duties](../changes/2026-10-18-duties/) (story [the message for the next game](../changes/done/2026-10-29-next-game-message/))

## Context
The coordinator writes messages by hand and copies them into the chat: before each game one for
the team and the parents (when, where, who drives, who has table or referee duty), and before each
weekend one for the referee committee. The tool must make them, correct the first time, from the
duties it already keeps (decision 0023). The screens are in English, the messages in Dutch. The
start page shows a link to a message only when it has something to say, so the link and the text
must never disagree (decision 0022).

## Decision
- **A message is made by one rule in the core** (`team_core::messages`, decision 0001), as plain text with the Dutch day and month names in the module. The server only passes in the stored games, their places (`game_places`) and the names (`display_names`). Exact-text tests fix each form.
- **Made at each request, from what is stored then.** Nothing about a message is stored: no copy, no "sent" state. The coordinator may edit the text after pasting.
- **The texts are compiled in.** There are no templates the coordinator can edit (an idea of 2026-10-10, out of scope); a change of wording is a change of code with its tests.
- **The link on the start page comes from the same function as the page** (`api::messages::next_game_text`), with the data `GET /api/start` already reads; `GET /api/messages/next-game` answers `{ "text": null }` when there is no message, a normal state, not 404.
- **Words follow `docs/language.md`**: "Thuis" is a game in the club's own hall (`game.home.club` is the club), as `required_duties` decides drivers, not the side of the team in the game list (the coordinator, 2026-10-29).

## Options considered
| Option | For | Against |
|--------|-----|---------|
| The text in the core (chosen) | One rule, exact-text tests, no I/O | Dutch words in the core |
| The text in the front end from the start page data | No new route | The start page has no drivers after next week; the rule would be in two places |
| Templates the coordinator can edit | Wording without a release | Storage, a screen and checks for wrong templates; out of scope |
| A flag on the start page from the same function (chosen) | One request; the link and the text cannot differ | One more field on `GET /api/start` |
| The start page asks the message route | No change of the start page answer | Two requests for one page |
| "Thuis" by the club's hall (chosen) | The word list; the same test as the drivers | Differs from the game list's home side for two club teams against each other |

## Consequences
- The message for the referee committee (story D) is a second function in the same module, with its own flag in `messages` on `GET /api/start`; the part "Messages" shows when either has something to say.
- A wrong word or space is a failing test, not a silent change.
- The messages always show what is stored now; a message copied earlier is not updated in the chat.
