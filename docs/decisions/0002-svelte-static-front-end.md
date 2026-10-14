# 0002 – Svelte static front end with shadcn-svelte

**Status**: valid
**Date**: 2026-10-10
**Change**: [2026-10-10-games-from-game-list](../changes/done/2026-10-10-games-from-game-list/)

## Context
The coordinator wants screens that look great on a phone and on a computer, and does not do
front-end work. The coordinator chose a separate front end over pages made by the server, and
asked for a nice and simple framework.

## Decision
- SvelteKit in static mode (built to plain files), Tailwind for styling, shadcn-svelte for ready-made parts. The front end lives in `web/`.
- The Rust server serves the built files, so one image goes to the cluster and there is no second server.
- The front end talks to the JSON API under `/api`. The server publishes an OpenAPI description (utoipa); the front end generates its types from it (openapi-typescript). A check fails when the committed types differ from the generated ones.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Pages made by the Rust server with htmx | One language, one build | The coordinator chose a separate front end |
| Svelte static app + shadcn-svelte | Fewest concepts; clean ready-made parts; known from the old app | Two languages and two builds |
| Vue | Simple, good kits | More concepts than Svelte |
| React | Largest set of libraries | Most code and choices for a small tool |

## Consequences
- Each feature has a Rust side and a Svelte side; the API contract is checked at build.
- The look (colours, fonts, light and dark mode) is set once and reused.
- Browser tests (Playwright on a phone-sized screen) cover the screens.
