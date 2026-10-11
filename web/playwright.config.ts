import { defineConfig } from '@playwright/test';

// The tests run against the real Rust server with the built front end (`npm run build` first).
// One server uses the debug-only login bypass (logged in as "Developer"), one does not.
const database = process.env.DATABASE_URL ?? 'postgresql://team:team@localhost:5432/team_v2';

function server(port: number, bypass: boolean) {
	return {
		command: 'cargo run --quiet --bin team -- serve',
		cwd: '..',
		url: `http://localhost:${port}/api/health/liveness`,
		reuseExistingServer: false,
		timeout: 300_000,
		env: {
			DATABASE_URL: database,
			OIDC_ISSUER_URL: 'https://id.example.test',
			OIDC_CLIENT_ID: 'team-coordinator',
			OIDC_CLIENT_SECRET: 'e2e',
			SESSION_SECRET: 'e2e-session-secret-of-at-least-32-chars',
			ORIGIN: `http://localhost:${port}`,
			HOST: '127.0.0.1',
			PORT: String(port),
			WEB_DIR: 'web/build',
			DEV_AUTH_BYPASS: String(bypass)
		}
	};
}

export default defineConfig({
	testDir: 'e2e',
	testMatch: '**/*.e2e.ts',
	outputDir: 'test-results',
	use: {
		viewport: { width: 390, height: 844 },
		deviceScaleFactor: 2,
		isMobile: true,
		hasTouch: true,
		launchOptions: { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH }
	},
	projects: [
		{ name: 'logged-in', testMatch: 'logged-in.e2e.ts', use: { baseURL: 'http://localhost:4100' } },
		{
			name: 'logged-out',
			testMatch: 'logged-out.e2e.ts',
			use: { baseURL: 'http://localhost:4101' }
		}
	],
	webServer: [server(4100, true), server(4101, false)]
});
