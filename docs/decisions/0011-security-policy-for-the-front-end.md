# 0011 – Security policy for the front end

**Status**: valid
**Date**: 2026-10-11
**Change**: [2026-10-10-the-base](../changes/done/2026-10-10-the-base/)

## Context
The server sends a Content-Security-Policy on every answer, so that a script from anywhere else
cannot run in the tool. The design had `default-src 'self'; frame-ancestors 'none'`. The built
SvelteKit app starts with one inline script in `index.html`, and SvelteKit sets a `style`
attribute on its page-change announcer for screen readers; that policy blocked both.

## Decision
- At start, the server reads `index.html` and computes the SHA-256 hash of each inline script. The policy is `default-src 'self'; script-src 'self' 'sha256-…'; style-src 'self' 'unsafe-inline'; frame-ancestors 'none'`.
- Only those exact inline scripts may run; any other inline script is blocked.
- Inline styles are allowed; styles cannot run code.
- The policy is fixed for the life of the process; a new front end comes with a new image and a restart.
- Next to it: `X-Content-Type-Options: nosniff` and `Referrer-Policy: same-origin`.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Script hash computed at start, inline styles allowed | Strict for scripts; no build step to keep in sync | Inline styles allowed |
| `'unsafe-inline'` for scripts too | Simple | Gives up the main protection |
| SvelteKit's own `kit.csp` (a policy tag in the page) | Made by SvelteKit | The page tag and the server header both apply, so the server header must still allow the script |
| Hash of the announcer style as well (`'unsafe-hashes'`) | No inline styles | Breaks when SvelteKit changes that style |

## Consequences
- A browser test fails on any console error, so a script or style the policy blocks is found before release.
- The front end must not load anything from other sites (fonts are packed with the app).
