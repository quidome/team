# MVP PRD

**Status:** Draft. Formalized from current discovery for the 2026–2027 season; unresolved items remain open rather than decided by this document.

## Purpose

Provide a private coordinator workspace for the U16-1 team that supports the current chat-heavy coordination workflow. Players do not use the app in the MVP; one authenticated coordinator manages current-season information and prepares content to copy into WhatsApp.

## Product context

- Current season: 2026–2027.
- Primary team: U16-1.
- Other teams and their games may appear in the shared program, but they are not independent workspaces.
- The backend is the shared source of truth across the coordinator's phone, tablet, and computer.

## Goals

The MVP must let the coordinator:

1. Maintain players and season-specific team memberships.
2. Manage the program of games and recurring training sessions.
3. Record participation and absence reasons.
4. Coordinate game-related duties and see fair-use guidance based on completed duties.
5. Generate editable Dutch WhatsApp poll drafts for manual copying.
6. Import program data without silently overwriting existing games.

## In scope

### Players and memberships

- Store player name, birthday, and association ID.
- Maintain a membership for a player, team, and season, including active status, jersey number, participation type, and primary or secondary relationship when applicable.
- Calculate the normal age group from season start year and birth year while allowing players to play up with another team.
- Exclude trains-only players from game-related duties.

### Program and events

- Maintain games, including other teams' games, and identify U16-1 games from configured team identity data.
- Represent games as stable fixtures with scheduled occurrences so cancellation and rescheduling retain their history.
- Create recurring training series and generated occurrences, with one-off additions, cancellations, rescheduling, and per-occurrence time or location overrides.
- Store reusable locations, game home/away status, travel time, and a default 30-minute arrival buffer with per-occurrence overrides.

### Participation

- Record training attendance and game participation.
- Support bulk selection, exceptions, and absence reasons.
- Retain history for later attendance and participation percentages; percentage denominators remain configurable until decided.

### Duties and fairness

- Configure referee and jury requirements for non-own games and driving slots for own away games.
- Record chat-originated sign-ups, including over-capacity volunteers, and preserve sign-up and reassignment history.
- Treat completion as the basis for duty balance; fairness is guidance, not enforcement.
- Retain cancelled or incomplete duties without counting them toward fairness.

### Coordinator workspace and output

- Provide Home, Program, Team, History, and Settings/Imports navigation areas.
- Show upcoming events, open duties, generated tasks, and manual reminders on Home.
- Maintain an activity and audit trail for meaningful changes.
- Generate editable Dutch home-game and away-game poll drafts, including optional transport/departure details, for manual WhatsApp copying.

### Imports

- Support manual game entry plus `.xls`, `.xlsx`, `.ods`, and CSV input.
- Provide column mapping, validation, preview, correction, import provenance, duplicate detection, and field-by-field reconciliation.
- Import all source games and classify U16-1 games through configured team identity.

## Non-goals and deferred work

- Player accounts, multi-user/team workspaces, and trainer-specific workflows.
- Previous-season reporting.
- Native iOS development, sideloading, offline synchronization, and direct WhatsApp integration.
- Email, SMS, and push notifications.
- Tracking adult fulfillers, cars, or vehicle capacity.
- Separate injury management and custom duty-type administration beyond generic extra duties.

## Acceptance criteria

- A coordinator can use the responsive PWA on an iPhone and a computer against the same backend data.
- A recurring training series generates occurrences through its end date without preventing individual cancellation, rescheduling, or additions.
- U16-1 games, other-team games, home/away status, duty requirements, driving slots, and departure suggestions can coexist in the program.
- Training-only players participate in training attendance but not game-duty assignment.
- Reassignment preserves sign-up and assignment history; only completed duties affect fairness guidance.
- Import conflicts require deliberate field-level resolution and never silently overwrite existing data.
- Dutch poll drafts are generated for manual copying rather than sent by the application.

## Technical and delivery constraints

Follow [[projects/team/architecture/technical-direction|Technical Direction]] and [[projects/team/planning/mvp-implementation-plan|MVP Implementation Plan]]. The MVP is a SvelteKit/TypeScript PWA candidate with PostgreSQL, Drizzle, Pocket ID OIDC, a backend-managed session, default-deny application APIs, Nix development tooling, and deployment to the existing home Kubernetes cluster. The stack remains subject to validation during the first implementation slice.

## Open decisions

- Identify the program-data source and its export, web, or API capabilities.
- Approve the exact Dutch poll templates and response wording.
- Decide the percentage denominators.
- Define any required game fields beyond teams, time, location, and competition details.
- Decide how possible-but-uncertain imported duplicates should be handled.
- Validate the working SvelteKit, TypeScript/Node.js, PostgreSQL, and Drizzle combination.
- Decide whether offline support, internal HTTPS/ingress details, or future multi-user roles are needed.

## Source notes

- [[projects/team/product/mvp-scope|MVP Scope]]
- [[projects/team/architecture/technical-direction|Technical Direction]]
- [[projects/team/planning/mvp-implementation-plan|MVP Implementation Plan]]
