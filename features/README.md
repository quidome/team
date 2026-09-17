# Behavior scenarios

Store user-visible behavior as Gherkin-style `.feature` files. Each scenario must have one or more automated tests that name the same behavior and run through `npm test`.

Write scenarios before their implementation. Keep them independent of SvelteKit, database, and identity-provider details; adapter-level tests cover those integrations separately.
