import { expect, test } from '@playwright/test';
import { expectAccessible, watchConsole } from './helpers';

test('without login, the start page offers to log in', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/');
	const login = page.getByRole('link', { name: 'Log in' });
	await expect(login).toBeVisible();
	await expect(login).toHaveAttribute('href', '/auth/login');
	await expect(page.getByRole('button', { name: 'Log out' })).toHaveCount(0);
	await expectAccessible(page);
	expect(errors).toEqual([]);
});

test('without login, the sample page asks to log in', async ({ page }) => {
	await page.goto('/sample');
	await expect(page.getByText('Log in to see the sample schedule.')).toBeVisible();
	await expect(page.getByTestId('game')).toHaveCount(0);
});

test('screenshot of the login page for the look', async ({ page }) => {
	for (const colorScheme of ['light', 'dark'] as const) {
		await page.emulateMedia({ colorScheme });
		await page.goto('/');
		await expect(page.getByRole('link', { name: 'Log in' })).toBeVisible();
		await page.screenshot({ path: `test-results/look/login-${colorScheme}.png`, fullPage: true });
	}
});
