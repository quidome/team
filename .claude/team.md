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
status: restart. The SvelteKit/TypeScript app in this repository is the old version. Use it and
  docs/changes/done/2026-09-20-mvp/ as reference, not as the design to keep.
data: PostgreSQL on the cluster, through sqlx with forward-only migrations (docs/decisions/0004).
structure: Rust workspace, crates/core without I/O, crates/server for I/O and commands; web/ is the
  Svelte front end (docs/decisions/0001, 0002). Until story 1 of the first change replaces it, the
  old app is still in the repository.
deployment: gitops-lab/applications/productivity/team (Argo CD, Helmfile, Vault). Never apply by hand.
check (old app): just check
dev-shell: nix develop path:.
