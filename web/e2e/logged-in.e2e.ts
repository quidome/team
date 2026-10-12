import { expect, test } from '@playwright/test';
import { expectAccessible, watchConsole } from './helpers';

test('after login, the header shows the name and a logout action', async ({ page }) => {
	// What the start page shows below the header depends on the saved choice (choose.e2e.ts).
	const errors = watchConsole(page);
	await page.goto('/');
	await expect(page.getByTestId('user-name')).toHaveText('Developer');
	await expect(page.getByRole('button', { name: 'Log out' })).toBeVisible();
	await expectAccessible(page);
	expect(errors).toEqual([]);
});

test('log out posts to the server and returns to the start page', async ({ page }) => {
	await page.goto('/');
	const [request] = await Promise.all([
		page.waitForRequest((request) => request.url().endsWith('/auth/logout')),
		page.getByRole('button', { name: 'Log out' }).click()
	]);
	expect(request.method()).toBe('POST');
	// The development user is always logged in, so the page shows the name again; that the
	// session really ends is tested in the Rust tests (logout_ends_the_session).
	await expect(page).toHaveURL('/');
	await expect(page.getByTestId('user-name')).toHaveText('Developer');
});

test('screenshots of the start page for the look', async ({ page }) => {
	// The schedule's screenshots are taken in choose.e2e.ts, while M16 1 is chosen.
	for (const colorScheme of ['light', 'dark'] as const) {
		await page.emulateMedia({ colorScheme });
		await page.goto('/');
		await page.waitForLoadState('networkidle');
		await page.screenshot({ path: `test-results/look/start-${colorScheme}.png`, fullPage: true });
	}
});
