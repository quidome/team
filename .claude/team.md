# Team for team

project-type: software          # software | infrastructure  (see project-types/)
maturity: early                 # restart: new backend, probably in Rust (to be decided in /vision)
product-docs: docs/product/

## Roles
# role: persona + expertise, expertise
product-lead: carl
product-designer: alice
architect: steve + rust
engineer: bruce + rust
tester: august + rust
developer: matthew + rust
scope-keeper: matthew

## Project notes
# Short facts every stage must know, for example the check command.
status: restart. The SvelteKit/TypeScript app in this repository is the old version. Use it and
  docs/changes/done/2026-09-20-mvp/ as reference, not as the design to keep.
data: the user is the only user. Most data is in a Google Sheet; the PostgreSQL database may not be needed (open question).
check (old app): just check
dev-shell: nix develop path:.
structure: the code structure (for example hexagonal) is not decided yet. The first /design proposes it
  and writes a decision record; add the matching expertise here after that.
