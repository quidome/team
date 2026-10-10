# MVP Implementation Plan
## Current implementation status
- **Complete:** Foundation, authentication, season/team configuration, player registration, season-specific memberships, reusable locations, recurring training generation, stable game fixtures and occurrences, participation persistence, and the combined program read model.
- **Complete:** Program management UI, recurring training series, one-off training occurrences, event cancellation, event rescheduling, and per-occurrence time/location overrides.
- **Complete:** Team and attendance views, bulk participation recording with absence reasons, participation history, configurable attendance percentage denominators (recorded or scheduled eligible occurrences), duties, transport, WhatsApp drafts, coordinator tasks, imports, worksheet selection, reconciliation, reports, audit logging, health checks, backup verification, and production hardening.
- **Complete:** Provider-neutral container packaging with separate runtime and migration images, runtime configuration documentation, and GitHub Actions GHCR publishing.
- **Delegated:** Production deployment to the separate GitOps repository, including the target host, PostgreSQL, Pocket ID, HTTPS/ingress, scheduled backups, and production environment values.

**Status:** MVP implementation is complete. Production deployment is tracked outside this repository.

**Post-MVP work (2026-09-20, not yet reflected in the product notes):** After this plan was marked complete, the same-day source history added three further changes: a rename of the Program navigation area and routes to Events (`/program` now redirects to `/events`); season halves (`H1`/`H2` per season) and external opponents as a concept distinct from own teams for game fixtures, carrying an opponent name, address, and travel time without full team/roster modeling; and a consolidated Admin screen (`/settings` now redirects to `/admin`) offering modal-based create/edit for seasons, teams, and locations in place of the earlier Settings page. These are observations from the implementation, not confirmed product decisions — the underlying intent (why external opponents and season halves are needed, and how they relate to the existing "other teams in the shared program" scope in [[projects/team/product/mvp-scope|MVP Scope]]) should be captured once clarified.

Related notes:
- [[projects/team/product/mvp-scope|MVP Scope]]
- [[projects/team/product/project-brief|Project Brief]]
- [[projects/team/architecture/technical-direction|Technical Direction]]
## Delivery principles

- Build for one coordinator first; do not implement player accounts or multi-user workflows yet.
- Keep the backend as the source of truth for all devices.
- Keep current-state records editable, while maintaining an append-only audit trail for meaningful changes.
- Prefer small vertical slices that can be used with the current season before adding broader automation.
- Use real data only after authentication and secure internal access are in place.

## Phase 0 — Foundation and technical decisions
- Add the pinned Nix `flake.nix` development shell before application implementation.
- Add a Just task runner with commands for development, tests, checks, formatting, linting, builds, database tasks, and local execution.
- Define the Gherkin-style test format and write the first red acceptance scenario before implementing the first vertical slice.
- Establish the default-deny API authentication policy and test the explicitly unauthenticated operational endpoints.

- Validate the working SvelteKit + TypeScript/Node.js, PostgreSQL, and Drizzle stack against the first vertical slice.
- Use the separate `team` source repository and keep implementation out of the Dev vault.
- Define configuration for the current season (2026–2027) and primary team (U16-1).
- Deploy a minimal backend, PWA shell, and central database in the cluster.
- Configure internal HTTPS, ingress, backups, and basic observability.
- Register the application as a Pocket ID OIDC client.
- Implement one coordinator login with a backend-managed secure session.

## Phase 1 — Core domain and data storage

Implement migrations and backend APIs for:

- Seasons, teams, reusable locations, and team identity values
- Players and season-specific team memberships
- Age-group calculation from season start year and birth year
- Event series and concrete event occurrences
- Stable game fixtures and scheduled game occurrences
- Participation records and absence reasons
- Duty requirements and individual duty slots
- Sign-ups, assignments, reassignment, and completion status
- Manual and generated tasks
- Audit entries and import provenance

Add seed data for the current season and U16-1.

## Phase 2 — Program and event management

- Add and edit games, including games involving other teams.
- Identify games involving U16-1 using configured team identity.
- Add recurring training series with end dates.
- Generate training occurrences.
- Support one-off occurrences, cancellation, rescheduling, and time/location overrides.
- Add reusable locations and travel-time data.
- Calculate suggested departure time from game time, travel time, and the 30-minute buffer.
- Build the Program view and the initial Home view.

## Phase 3 — Players and participation

- Build the Team view for player and membership management.
- Calculate and display normal age groups while allowing team memberships that play up.
- Add training attendance and game participation screens.
- Support select all, deselect all, absence reasons, and Store.
- Add training and game participation history.
- Implement initial attendance and participation percentages, keeping denominator choices configurable.

## Phase 4 — Duties and transport

- Configure per-game referee and jury requirements from 0–2 slots each.
- Configure driving slots for own away games.
- Record sign-ups from chat, including over-capacity volunteers.
- Confirm assignments, change assignments, and preserve signup/assignment history.
- Automatically mark assignments completed after the occurrence, with manual correction.
- Add duty history and fairness views based on completed assignments.

## Phase 5 — WhatsApp drafts and coordinator tasks

- Add editable English UI and Dutch message templates.
- Generate home-game attendance poll drafts.
- Generate away-game attendance and transport poll drafts.
- Support optional departure-location text and per-game overrides.
- Add generated tasks and manual reminders to the Home view.
- Keep sending and duty-poll generation out of scope.

## Phase 6 — Import and reconciliation

- Add manual game entry alongside import workflows.
- Support `.xls`, `.xlsx`, `.ods`, and CSV input.
- Add column mapping, validation, preview, and correction.
- Import all source games and classify U16-1 games.
- Detect duplicates and conflicting parameters.
- Provide field-by-field existing-versus-imported choices and create merged records.
- Store import source and timestamp.

## Phase 7 — Reports, audit, and hardening

- Add current-season history views for participation and duties.
- Show attendance and game-participation percentages.
- Show duty counts and balance by player, duty type, and period.
- Make audit entries inspectable from relevant records.
- Test cancellation, rescheduling, reassignment, and correction behavior.
- Add database backups, access controls, error handling, and deployment documentation.

## Acceptance checks

- A recurring Tuesday training series generates occurrences through its end date.
- An individual occurrence can be cancelled, rescheduled, or added without damaging the series.
- A 14:30 own home game, 16:00 referee duty, and 17:30 referee/jury duty can be represented together.
- An own away game can generate a configurable number of driving slots and a departure suggestion.
- Training-only players appear for training attendance but not game-related duties.
- Multiple volunteers can be retained while only the selected player receives a completed-duty credit.
- Reassigning a duty preserves the original sign-up and assignment history.
- Only completed duties affect fairness calculations.
- Import conflicts can be resolved per field without silent overwrites.
- Dutch game-poll drafts can be copied manually from the PWA.
- Data is shared between an iPhone PWA and a computer through the central backend.

## Deferred work

- Player accounts and multi-user/team support
- Trainer workflows
- Previous-season views
- Native iOS app
- Direct WhatsApp integration
- Notifications and offline synchronization
- Vehicle and adult-fulfiller tracking
- Custom duty-type administration