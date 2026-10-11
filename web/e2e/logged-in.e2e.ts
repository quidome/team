import { expect, test } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

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

test.describe('sample page', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/sample');
	});

	test('groups games by date, with the team marked', async ({ page }) => {
		const headings = page.getByRole('heading', { level: 2 });
		await expect(headings).toHaveText(['Sat 17 Oct', 'Sat 24 Oct', 'Sat 31 Oct', 'Sat 7 Nov']);
		await expect(page.getByTestId('game')).toHaveCount(8);
		await expect(page.locator('[data-testid="game"][data-of-team="true"]')).toHaveCount(3);
		await expect(page.getByTestId('changes-not-seen')).toHaveText('2 changes not seen');
		await expect(page.getByText('time not known yet')).toBeVisible();
	});

	test('the filter shows games of the team or possible duty games', async ({ page }) => {
		await page.getByRole('radio', { name: 'Games of the team' }).click();
		await expect(page.getByTestId('game')).toHaveCount(3);
		await expect(page.locator('[data-testid="game"][data-of-team="false"]')).toHaveCount(0);

		await page.getByRole('radio', { name: 'Possible duty games' }).click();
		await expect(page.getByTestId('game')).toHaveCount(5);
		await expect(page.locator('[data-testid="game"][data-of-team="true"]')).toHaveCount(0);

		await page.getByRole('radio', { name: 'All' }).click();
		await expect(page.getByTestId('game')).toHaveCount(8);
	});

	test('a game that left the game list is crossed out', async ({ page }) => {
		const gone = page.getByTestId('game').filter({ hasText: 'no longer in the game list' });
		await expect(gone).toHaveCount(1);
		const decoration = await gone
			.locator('p')
			.first()
			.evaluate((element) => getComputedStyle(element).textDecorationLine);
		expect(decoration).toBe('line-through');
	});

	test('is accessible, has no console errors and fits a small phone', async ({ page }) => {
		const errors = watchConsole(page);
		await page.reload();
		await expect(page.getByTestId('game')).toHaveCount(8);
		await expectAccessible(page);
		await expectNoHorizontalScroll(page, 360);
		expect(errors).toEqual([]);
	});

	test('screenshots for the look', async ({ page }) => {
		for (const colorScheme of ['light', 'dark'] as const) {
			await page.emulateMedia({ colorScheme });
			for (const [name, path] of [
				['start', '/'],
				['sample', '/sample']
			]) {
				await page.goto(path);
				await page.waitForLoadState('networkidle');
				await page.screenshot({
					path: `test-results/look/${name}-${colorScheme}.png`,
					fullPage: true
				});
			}
		}
	});
});
