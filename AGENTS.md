# Team Project Agent Context

This repository contains the `team` project (a team coordinator tool).

## Status

The project started over in October 2026. The old SvelteKit/TypeScript app is removed; its
history is in `docs/changes/done/2026-09-20-mvp/` and its behaviour scenarios are in `features/`
(reference only). The new product is a Rust backend with a Svelte front end; see `docs/vision.md`
and `docs/decisions/`.

## Project knowledge

Product, architecture and planning knowledge lives in this repository under `docs/`. The notes
that used to be in the Obsidian vault (`projects/team/` in the Dev vault) are copied as history
to `docs/changes/done/2026-09-20-mvp/`; read them only when asked or when a stage needs the
history.

Do not invent product requirements or technical decisions when the documents leave them open.

## Way of working
This project uses the product way of working (skill `product-workflow`).
Team and roles: `.claude/team.md`. Word list: @docs/language.md
Living documents: docs/vision.md, docs/architecture.md, docs/product/. Open changes: docs/changes/.
Do not read docs/changes/done/ unless asked.
