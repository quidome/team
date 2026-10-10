# Technical Direction

**Status:** Initial direction; subject to validation during implementation.

## Architecture and development workflow

- Use hexagonal architecture. Keep the domain and application core independent of SvelteKit, Node.js, PostgreSQL, Pocket ID, import formats, and WhatsApp output.
- Use ports for persistence, authentication/session handling, time, imports, and message generation. Keep framework and infrastructure concerns in adapters and the composition root.
- Use test-driven development with a red/green/refactor cycle.
- Start each vertical slice with Gherkin-style behavior scenarios, then implement the smallest change that makes the scenario pass.

## Development environment
- The implementation lives in the separate `team` Git repository; the Dev vault contains project knowledge, not source code.

- Development runs in Nix.
- Provide a pinned `flake.nix` with a `devShells.default` containing the Node.js/npm toolchain, Just, and required development utilities.
- Keep JavaScript dependencies in the project lockfile and system/toolchain dependencies in the Nix flake.
- Use Just as the task runner for development, testing, formatting, linting, builds, database tasks, and local execution.

## API exposure policy

- Application and data endpoints are authenticated by default and deny access without a valid coordinator session.
- Explicit unauthenticated exceptions are limited to liveness, readiness, Prometheus metrics, OIDC login/callback/logout, and static PWA assets.
- Health endpoints return minimal operational information. Metrics are server-side/internal, must not expose personal data, and should not be reachable through the public application ingress.
- Add tests proving that unlisted API routes return `401` without a valid session.

## Testing layers

- Keep Gherkin scenarios as the behavioral specification for user-visible workflows.
- Test domain and application behavior with in-memory adapters.
- Test infrastructure adapters against real services where needed, especially PostgreSQL and OIDC boundaries.
- Keep a small set of browser-level tests for critical PWA flows.

## Client
- Working stack: SvelteKit full-stack application with TypeScript/Node.js, PostgreSQL, and Drizzle ORM; validate this choice during the first implementation slice.

- Build a responsive Progressive Web App (PWA) that works well on phones, tablets, and computers.
- The PWA should be installable from iOS and pinnable to the home screen, without requiring a native iOS app or sideloading.
- Offline support is not decided yet.

## Hosting and data

- Run the backend in the existing home Kubernetes cluster.
- Store shared team data in a central database so the same games, players, assignments, and history are available from every device.
- Keep the application internal to the home network or VPN initially; internet exposure may be considered later.

## Authentication

- Use Pocket ID as the OpenID Connect (OIDC) identity provider.
- The application backend will act as the OIDC client and handle the authorization callback and token validation.
- Prefer a backend-managed session with a secure, HTTP-only cookie; do not expose OIDC client secrets or long-lived tokens to the PWA.
- The MVP has one authenticated coordinator role; players do not log in. Keep future multi-user authorization as an extension point.
- Authentication should be in place before real team data is used, even while the application remains internal.

## Deployment considerations

- Use a stable HTTPS hostname, including for internal access where practical.
- Register the exact application callback URL in Pocket ID.
- The iOS authentication handoff may briefly open Safari or another browser context; test the installed-PWA behavior on the target iOS version.

## Open questions

- Validate the working SvelteKit/TypeScript/Node.js, PostgreSQL, and Drizzle stack during implementation.
- Is offline support needed?
- What is the preferred internal HTTPS and ingress setup?
- What roles and permissions would be needed if multi-user support is added later?