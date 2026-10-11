import { defineConfig } from '@playwright/test';
import { e2eDatabase } from './e2e/database';

// The tests run against the real Rust server with the built front end (`npm run build` first).
// One server uses the debug-only login bypass (logged in as "Developer"), one does not. Both use
// a fresh database (`e2e/global-setup.ts`) and a fake game list source (`e2e/fake-source.mjs`).
const fakeSourcePort = 4102;

function server(port: number, bypass: boolean) {
	return {
		command: 'cargo run --quiet --bin team -- serve',
		cwd: '..',
		url: `http://localhost:${port}/api/health/liveness`,
		reuseExistingServer: false,
		timeout: 300_000,
		env: {
			DATABASE_URL: e2eDatabase,
			// Compile from the query data in .sqlx/: the fresh database may not exist yet.
			SQLX_OFFLINE: 'true',
			OIDC_ISSUER_URL: 'https://id.example.test',
			OIDC_CLIENT_ID: 'team-coordinator',
			OIDC_CLIENT_SECRET: 'e2e',
			SESSION_SECRET: 'e2e-session-secret-of-at-least-32-chars',
			ORIGIN: `http://localhost:${port}`,
			HOST: '127.0.0.1',
			PORT: String(port),
			WEB_DIR: 'web/build',
			DEV_AUTH_BYPASS: String(bypass),
			GAME_LIST_URL: `http://127.0.0.1:${fakeSourcePort}`
		}
	};
}

export default defineConfig({
	testDir: 'e2e',
	testMatch: '**/*.e2e.ts',
	outputDir: 'test-results',
	globalSetup: './e2e/global-setup.ts',
	use: {
		viewport: { width: 390, height: 844 },
		deviceScaleFactor: 2,
		isMobile: true,
		hasTouch: true,
		launchOptions: { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH }
	},
	projects: [
		{
			// The choice is saved in the shared database: these run first, and the other logged-in
			// tests wait for them. The logged-out tests do not touch the choice.
			name: 'choose',
			testMatch: 'choose.e2e.ts',
			use: { baseURL: 'http://localhost:4100' }
		},
		{
			name: 'logged-in',
			testMatch: 'logged-in.e2e.ts',
			dependencies: ['choose'],
			use: { baseURL: 'http://localhost:4100' }
		},
		{
			name: 'logged-out',
			testMatch: 'logged-out.e2e.ts',
			use: { baseURL: 'http://localhost:4101' }
		}
	],
	webServer: [
		{
			command: 'node e2e/fake-source.mjs',
			url: `http://127.0.0.1:${fakeSourcePort}/health`,
			reuseExistingServer: false,
			env: { FAKE_SOURCE_PORT: String(fakeSourcePort) }
		},
		server(4100, true),
		server(4101, false)
	]
});
