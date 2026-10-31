# Messages

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
The tool makes the messages the coordinator used to write by hand, as Dutch text to copy into the
chat. The tool never sends them. Messages announce what comes in this week. Now: the message of
the week, for the team and the parents, and the committee message, for the referee committee. (The
duty overviews come in the next story of duties.)

## Behaviour
- **Where.** The [start page](start-page.md) has a part "Messages" below "Next" with the button "Message of the week", when this week (Monday to Sunday, Dutch time) has something to announce: a game of the team today or later, or table or referee places of the team. A past place that stayed open does not count for this. Otherwise the part is not shown.
- **The page** (`/messages/week`) shows the text, made from what is stored at that moment, and "Copy". "Copy" puts the text on the clipboard and says "Copied". When the browser refuses, the text stays shown with "Select the text and copy it."; a tap on the text selects all of it. "Back" returns to the start page. Opened without a message (for example from an old link): "No message this week."
- **The text**, for example:

  ```
  Week 26 oktober – 1 november

  Wedstrijd: zaterdag 31 oktober, 14:30
  Uit tegen B.Z. '72 M16 1
  Sporthal Bulwijk, Woerden
  Rijden: Anna, Bram, Cas, nog 1 open

  Tafel en scheidsrechter:
  za 31 okt 16:00 Blue Drakes X12 1 – De Dunckers X12 1
    Scheidsrechter: Gijs
    Tafel: Eva, nog 1 open
  ```

  - **The heading** has the Monday and the Sunday of the week that holds today, always both in full ("Week 12 oktober – 18 oktober", "Week 28 december – 3 januari"). On a Sunday it is still this week.
  - **A block "Wedstrijd:"** for each game of the team from today to Sunday that is still in the game list, in time order, with one empty line between blocks: the date and start time ("tijd nog niet bekend" when not known), "Thuis" for a game in the club's own hall (also when the team is the away side against another club team) or "Uit", the opponent, the hall and the town. A game today stays all day; a game before today is not in it. The court is not in it. The coordinator puts a poll under the message for each game, by hand.
  - **"Rijden:"**, only when the game needs drivers of the team and their number is above 0: the drivers by first name in the order of the places, then the open places ("Anna, Bram, nog 1 open"; "nog 3 open").
  - **"Tafel en scheidsrechter:"**: the games of the whole week with table or referee places of the team, also days that have passed, in time order with one empty line between games. Each with the short date, the start time (or ", tijd nog niet bekend,") and the two teams, then "Scheidsrechter:" and "Tafel:" with the players in place order and the open places. A kind without a place of the team at that game has no line. Places of other club teams are never mentioned. No hall: these places are always in the club's own hall.
  - **Past places** show the player who did it after a correction, and "niemand" for "Nobody" and for a place that stayed open.
  - Only places that count are in it ([duties](duties.md)): a coming game that left the game list, or that is no longer a home game of the club, is not. A past game keeps what is stored.
  - Two players with the same first name show the first letter of the last name ("Anna B.", "Anna V."), as on the other pages.
  - Without a game of the team from today to Sunday the text has the heading and the places; without places it has no second part.
- **The committee message** is the second button in "Messages", whenever a team is chosen for the current season, also in a week without places. The page (`/messages/committee`) works the same, with "Copy" and "Back"; without a team it says "No message: no team chosen." The text covers the same week:

  ```
  Tafel en scheidsrechters Blue Drakes M16 1, week 26 oktober – 1 november

  za 31 okt 13:00 Blue Drakes X12 1 – De Dunckers X12 1
    Scheidsrechter: Eva de Vries
    Tafel: Fleur Jansen, Gijs Bakker

  zo 1 nov 11:00 Blue Drakes M14 2 – Lions M14 1
    Scheidsrechter: nog 1 open
    Tafel: Hugo Smit
  ```

  The games and the lines follow the same rules as "Tafel en scheidsrechter:" above, with first and last names (the first name only when there is no last name). A past place that stayed open shows "niemand", also when nothing else is in the week. Without places: the heading and "Geen tafel- of scheidsrechtersdiensten deze week."

## Errors and limits
- The messages cover the current season only: a week around 1 July shows only the games of that season.
- Two players with the same full name are not told apart in the committee message.
- One week only. On a Sunday evening the next week cannot be announced yet (an idea in `docs/ideas.md`).
- "Copy" is checked in the browser tests; the check on the coordinator's phone after the release is in `docs/backlog.md`.
- The text has no departure time and cannot be changed in the tool; the coordinator edits it after pasting when needed.
- Trainings are never in a message.

## Interface
- Screens: `/messages/week`, `/messages/committee`.
- Server address (needs a login): `GET /api/messages/week`: `{ "text": "…" }`, or `{ "text": null }` when there is no message. `GET /api/start` has `messages.week` from the same rule. `GET /api/messages/committee`: the same form, `null` only without a chosen team; `messages.committee` from the same rule.
- The rules are `team_core::week_message` (with `week_of`) and `team_core::committee_message` (with `full_names`); the server calls each in one place (`api::messages::week_text`, `committee_text`). Decisions 0024, 0025 and 0026 record the choices.
