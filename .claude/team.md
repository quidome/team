# Team for team

project-type: software          # software | infrastructure  (see project-types/)
maturity: early                 # restart: Rust backend, Svelte front end (docs/decisions/0001, 0002)
product-docs: docs/product/

## Roles
# role: persona + expertise, expertise
product-lead: carl
product-designer: alice
architect: steve + rust, hexagonal, kubernetes
engineer: bruce + rust, typescript-svelte
tester: august + rust, typescript-svelte
developer: matthew + rust, typescript-svelte, nix
scope-keeper: matthew

## Project notes
# Short facts every stage must know, for example the check command.
status: restart. The old SvelteKit app is removed (story 1 of the first epic). Use
  docs/changes/done/2026-09-20-mvp/ and features/ as reference, not as the design to keep.
data: PostgreSQL on the cluster, through sqlx with forward-only migrations (docs/decisions/0004).
structure: Rust workspace, crates/core without I/O, crates/server for I/O and commands; web/ is the
  Svelte front end (docs/decisions/0001, 0002).
deployment: gitops-lab/applications/productivity/team (Argo CD, Helmfile, Vault). Never apply by hand.
check: just check (in the Nix shell; database tests need `just db-up` and DATABASE_URL)
dev-shell: already loaded (`nix develop` from the project directory); never `nix develop path:.`
