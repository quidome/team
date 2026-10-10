# MVP Scope

**Status:** Draft discovery scope for the 2026–2027 season. This is not a final PRD.

## Product intent

A private coordinator tool for managing the U16-1 team. It should make the current chat-based workflow easier without requiring players to become app users.

## MVP context

- Current season: 2026–2027.
- Primary team: U16-1.
- One authenticated coordinator uses the app.
- Other teams and games can appear in the program, but they are not separate workspaces in the MVP.
- Previous-season reporting is deferred.

## Players and teams

- Store player name, birthday, and association ID.
- Store team membership per player and season.
- Membership includes active/inactive status, jersey number, participation type (`trains only` or `trains and plays`), and primary/secondary relationship where needed.
- Calculate the normal age group algorithmically from the season’s starting year and birth year: assign the next even age group above the player’s age in the season’s starting year. For 2026–2027, birth year 2011 maps to U16 and birth year 2010 maps to U18; each supported age group covers two birth years.
- Players may participate with another team, such as U18-1, while remaining primarily associated with U16-1.
- Trains-only players are excluded from game-related duties.

## Program and events

- Keep all relevant games in one program, including games involving other teams.
- Identify games involving U16-1 using configured team identity data.
- Model a game as a stable fixture with one or more scheduled occurrences, so cancellations and rescheduling do not erase the original planned occurrence.
- Support recurring training series with a start date, end date, weekday, time, duration, and location.
- Generate individual training occurrences and support one-off additions, cancellation, and per-occurrence time or location changes.
- A game or training occurrence can reference a reusable location.
- Games include home/away status, location, travel time, and a 30-minute default arrival buffer. The buffer and departure details can be overridden per occurrence.
- Home-court allocation is not tracked as a separate feature.

## Participation

- Training occurrences record attendance only.
- Game occurrences initially treat presence and playing as the same status.
- Support a bulk workflow: select all, deselect exceptions, enter absence reasons, and store.
- Absence reasons may include injury, illness, or other reasons. Injury is not a separate medical subsystem.
- Retain participation history so training and game percentages can be calculated later.

## Duties

- Track all duties, including referee, jury, driving, extra duty, and future types.
- For a non-own game, record our team’s required contribution as 0–2 referees and 0–2 jurors.
- An own home game has no driving requirement. An own away game has a configurable number of driving-duty slots, typically 2–3.
- Duties are assigned to players. Actual adult fulfillers, cars, and vehicle capacities are not tracked in the MVP.
- Players can sign up in chat; the coordinator records sign-ups in the app.
- Keep all sign-ups, including volunteers who are not selected. Support over-capacity volunteers as not selected or waitlisted.
- Assignments can be changed. The final player receives completion credit; the original sign-up and assignment history remain visible.
- An assignment is assumed completed after its occurrence unless changed. Cancelled or incomplete duties remain logged but do not count toward fairness.

## History, balance, and tasks

- Keep an activity log of player participation and duties linked to the relevant game or training occurrence.
- Derive attendance percentages, game participation percentages, and duty balance from the log rather than storing counters as the source of truth.
- Use completed duties for fairness calculations; fairness is guidance, not enforcement.
- Keep an audit trail of meaningful changes, including old and new values, actor, and timestamp.
- The home screen combines upcoming events, open duties, generated tasks, and manual reminders.
- No email, SMS, or push notifications are required for the MVP.

## WhatsApp output

- Generate Dutch WhatsApp poll drafts while the app UI and documentation remain in English.
- Home-game polls cover attendance.
- Away-game polls cover attendance and transport.
- Referee and jury duties remain manually coordinated in chat for the MVP.
- Message wording, emojis, departure-location inclusion, and date formatting should be editable templates.
- The app generates content for manual copying; it does not send WhatsApp messages or create polls directly.

## Imports

- Support manual entry and imports from Excel (`.xls`/`.xlsx`), OpenDocument spreadsheets (`.ods`), and CSV.
- Import all games from a source and identify our games using configured team identity.
- Never silently overwrite existing games.
- Detect duplicate or likely-identical games and show conflicting parameters side by side.
- Allow a field-by-field choice between existing and imported values, producing a merged record.
- Preview, column mapping, validation, correction, and import provenance are required.
- Whether the current iOS app provides an export, web interface, or API remains open.

## Initial client and navigation

- Responsive PWA for mobile, tablet, and computer.
- It can be added to an iOS home screen; no native iOS app is required.
- Home, Program, Team, History, and Settings/Imports are the initial navigation areas.
- Only the coordinator uses the app; player sign-up remains external to the app.

> Note (2026-09-20, implementation-driven, not yet confirmed as scope): the source repository has since renamed the Program area to Events and the Settings area to Admin, and introduced external opponents and season halves for game fixtures. See the post-MVP note in [[projects/team/planning/mvp-implementation-plan|MVP Implementation Plan]]. This scope document has not been updated to match; treat the rename and the opponent/season-half concepts as unreviewed until reconciled here.

## Initial technical direction

- Run the backend in the existing home Kubernetes cluster.
- Store shared data in a central database.
- Use Pocket ID as the OIDC provider, with a backend-managed session.
- Keep the app internal to the home network or VPN initially and require authentication before real team data is used.
- See [[projects/team/architecture/technical-direction|Technical Direction]] for deployment and authentication details.

## Explicitly deferred

- Player accounts and multi-user/team workspaces
- Trainer-specific functionality
- Previous-season reporting
- Native iOS development and sideloading
- Direct WhatsApp integration and duty polls
- Email, SMS, and push notifications
- Offline synchronization
- Tracking adult fulfillers, cars, or vehicle capacity
- Separate injury management
- Custom duty-type administration beyond the generic extra-duty type

## Open questions

- What app or association provides the program data, and does it offer export or API access?
- What exact Dutch poll templates and response wording should be used?
- Should percentage denominators use all scheduled games/sessions or only expected/selected ones?
- What game fields are required beyond teams, time, location, and competition details?
- What should happen when an imported record is a possible duplicate but not confidently identical?
- Validate the working SvelteKit + TypeScript/Node.js, PostgreSQL, and Drizzle stack during the first implementation slice.
- Why does a game fixture need an external opponent (name, address, travel time) distinct from an own team, and why group games into season halves? Confirm whether this replaces or extends the "other teams in the shared program" scope above.