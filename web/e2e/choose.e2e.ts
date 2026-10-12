// Choosing the club and team (story 2). The tests share one fresh database and run in order:
// each one starts from the choice the one before saved. They have their own Playwright project,
// which runs before the other logged-in tests; the logged-out tests do not touch the choice. The fake source (fake-source.mjs) knows
// Blue Drakes' games and gives every other club a season with no games.
import { expect, test } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

test.describe.configure({ mode: 'serial' });

test('the first visit shows only the choose step', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/');
	await expect(page.getByRole('heading', { name: 'Choose your club and team' })).toBeVisible();
	await expect(page.getByLabel('Club name')).toBeVisible();
	await expect(page.getByRole('link', { name: 'See the sample schedule' })).toHaveCount(0);
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([]);
});

test('a search with no club says so, and the next search works', async ({ page }) => {
	await page.goto('/');
	await page.getByLabel('Club name').fill('zzz');
	await page.getByRole('button', { name: 'Search' }).click();
	await expect(page.getByText('No club found. Try another name.')).toBeVisible();

	await page.getByLabel('Club name').fill('drakes');
	await page.getByRole('button', { name: 'Search' }).click();
	await expect(page.getByTestId('club-result')).toHaveText(['Blue Drakes Woerden']);
});

test('the coordinator finds the club and chooses the team', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/');
	await page.getByLabel('Club name').fill('drakes');
	await page.getByRole('button', { name: 'Search' }).click();
	await page.getByTestId('club-result').filter({ hasText: 'Blue Drakes' }).click();

	const teams = page.getByTestId('team-choice');
	await expect(teams).toHaveText(['M16 1', 'M16 2', 'M18 1', 'MSE 1', 'X12 1', 'X14 1']);
	await expectAccessible(page);
	await teams.filter({ hasText: 'M16 1' }).click();

	await expect(page.getByTestId('chosen-team')).toHaveText('Blue Drakes M16 1');
	await expect(page.getByRole('link', { name: 'Change' })).toBeVisible();
	await expect(page.getByRole('link', { name: 'See the sample schedule' })).toBeVisible();
	await expectAccessible(page);
	expect(errors).toEqual([]);
});

test('the coordinator reads the game list', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/');
	await page.getByRole('button', { name: 'Read the game list' }).click();
	await expect(page.getByTestId('reading-result')).toHaveText('71 games read');
	await expect(page.getByTestId('last-read')).toHaveText(
		/^Last read: \w{3} \d{1,2} \w+ \d{4} \d{2}:\d{2}$/
	);
	await expect(page.getByRole('button', { name: 'Read the game list' })).toBeEnabled();
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([]);
});

test('a second reading of the same game list has no changes', async ({ page }) => {
	await page.goto('/');
	await expect(page.getByTestId('last-read')).toBeVisible();
	await expect(page.getByTestId('reading-result')).toHaveCount(0);
	await page.getByRole('button', { name: 'Read the game list' }).click();
	await expect(page.getByTestId('reading-result')).toHaveText('No changes');
});

test('the coordinator changes the team later', async ({ page }) => {
	await page.goto('/');
	await page.getByRole('link', { name: 'Change' }).click();
	await expect(page).toHaveURL('/choose');
	await expect(page.getByRole('heading', { name: 'Blue Drakes' })).toBeVisible();
	await page.getByTestId('team-choice').filter({ hasText: 'M16 2' }).click();

	await expect(page).toHaveURL('/');
	await expect(page.getByTestId('chosen-team')).toHaveText('Blue Drakes M16 2');
});

test('a club with no games yet can be saved without a team', async ({ page }) => {
	await page.goto('/choose');
	await page.getByRole('button', { name: 'Other club' }).click();
	await page.getByLabel('Club name').fill('green giants');
	await page.getByRole('button', { name: 'Search' }).click();
	await page.getByTestId('club-result').filter({ hasText: 'Green Giants' }).click();

	await expect(page.getByText(/Green Giants has no games for \d{4}-\d{4} yet/)).toBeVisible();
	await expect(page.getByTestId('team-choice')).toHaveCount(0);
	await page.getByRole('button', { name: 'Save club' }).click();

	await expect(page).toHaveURL('/');
	await expect(page.getByRole('heading', { name: 'Choose your team' })).toBeVisible();
	await expect(page.getByRole('heading', { name: 'Green Giants' })).toBeVisible();
	await expect(
		page.getByText(/Green Giants has no games for \d{4}-\d{4} yet\. The club is kept/)
	).toBeVisible();
	await expect(page.getByRole('button', { name: 'Save club' })).toHaveCount(0);

	// The page for a change shows the kept club in the same way.
	await page.goto('/choose');
	await expect(page.getByText(/The club is kept/)).toBeVisible();
	await expect(page.getByRole('button', { name: 'Save club' })).toHaveCount(0);
	await expect(page.getByRole('link', { name: 'See the sample schedule' })).toHaveCount(0);
});
