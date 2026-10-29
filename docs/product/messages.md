# Messages

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
The tool makes the messages the coordinator used to write by hand, as Dutch text to copy into the
chat. The tool never sends them. Now: the message for the next game, for the team and the parents.
(The message for the referee committee comes in the next story of duties.)

## Behaviour
- **Where.** The [start page](start-page.md) has a part "Messages" below "Next" with the link "Message for the next game", when that message has something to say: a coming game of the team (today or later), or table or referee places of the team in the coming week. Otherwise the part is not shown.
- **The page** (`/messages/next-game`) shows the text, made from what is stored at that moment, and "Copy". "Copy" puts the text on the clipboard and says "Copied". When the browser refuses, the text stays shown with "Select the text and copy it."; a tap on the text selects all of it. "Back" returns to the start page. Opened without a message (for example from an old link): "No message: no coming game and no duties this week."
- **The text**, for example:

  ```
  Volgende wedstrijd: zaterdag 31 oktober, 14:30
  Uit tegen B.Z. '72 M16 1
  Sporthal Bulwijk, Woerden

  Rijden: Anna, Bram, Cas (nog 1 open)

  Tafel en scheidsrechter deze week:
  za 31 okt 16:00 Blue Drakes X12 1 – De Dunckers X12 1
    Tafel: Eva, nog 1 open
    Scheidsrechter: Gijs
  ```

  - **The next game**: the first coming game of the team that is still in the game list, also when it is weeks away; a game today stays the next game all day. Its date and start time ("tijd nog niet bekend" when not known), "Thuis" for a game in the club's own hall (also when the team is the away side against another club team) or "Uit", the opponent, the hall and the town. The court is not in it.
  - **"Rijden"**, only when the game needs drivers of the team and their number is above 0: the drivers by first name in the order of the places, the open places as "(nog 1 open)", or "nog 3 open" when none is filled.
  - **"Tafel en scheidsrechter deze week"**: the games from today until the coming Sunday (on a Sunday: that day only) with table or referee places of the team, whether or not the team plays that week. Each with the short date, the start time (or ", tijd nog niet bekend,") and the two teams, then "Tafel:" and "Scheidsrechter:" with the planned players in place order and the open places ("Eva, nog 1 open", "nog 2 open"). A kind without a place of the team at that game has no line.
  - Only places that count are in it ([duties](duties.md)): a coming game that left the game list, or that is no longer a home game of the club, is not.
  - Two players with the same first name show the first letter of the last name ("Anna B.", "Anna V."), as on the other pages.
  - Without a next game the text has only the week part; without places this week it has no week part.

## Errors and limits
- The message covers the current season only.
- On a Sunday the week part holds that Sunday only (an idea in `docs/ideas.md` for messages made on Sunday evening).
- "Copy" is checked in the browser tests; the check on the coordinator's phone after the release is in `docs/backlog.md`.
- The text has no departure time and cannot be changed in the tool; the coordinator edits it after pasting when needed.

## Interface
- Screen: `/messages/next-game`.
- Server address (needs a login): `GET /api/messages/next-game`: `{ "text": "…" }`, or `{ "text": null }` when there is no message. `GET /api/start` has `messages.next_game` from the same rule.
- The rule is `team_core::next_game_message` (with `coming_week`); the server calls it in one place (`api::messages::next_game_text`). Decision 0024 records the choices.
