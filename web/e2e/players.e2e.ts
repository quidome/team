// Players and settings (attendance at training, story A). The server's clock is 12 October 2026
// (DEV_NOW), so the season is 2026-2027. The tests share one database and run in this order.
import { expect, test } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

test.describe.configure({ mode: 'serial' });

test('the start page links to players and settings', async ({ page }) => {
	await page.goto('/');
	const links = page.getByTestId('team-links');
	await expect(links.getByRole('link', { name: 'Players' })).toBeVisible();
	await expect(links.getByRole('link', { name: 'Settings' })).toBeVisible();
});

test('add a player with a first name only', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/players');
	await expect(page.getByRole('heading', { name: 'Players' })).toBeVisible();
	await page.getByRole('button', { name: 'Add player' }).click();
	const form = page.getByTestId('add-player');
	await form.getByLabel('First name').fill('Quinn');
	await form.getByLabel('Only trains').check();
	await expect(form.getByLabel('In the team from')).toHaveValue('2026-10-12');
	// Only dates of the current season.
	await expect(form.getByLabel('In the team from')).toHaveAttribute('min', '2026-07-01');
	await expect(form.getByLabel('In the team from')).toHaveAttribute('max', '2027-06-30');
	await form.getByRole('button', { name: 'Save' }).click();
	const player = page.getByTestId('player').filter({ hasText: 'Quinn' });
	await expect(player).toBeVisible();
	await expect(player).toContainText('Only trains');
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([]);
});

test('the same name again shows a warning', async ({ page }) => {
	await page.goto('/players');
	await page.getByRole('button', { name: 'Add player' }).click();
	const form = page.getByTestId('add-player');
	await form.getByLabel('First name').fill('quinn');
	await form.getByRole('button', { name: 'Save' }).click();
	await expect(page.getByTestId('same-name')).toHaveText(
		'There is already a player with this name.'
	);
});

test('a player leaves the team, and it is undone', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/players');
	await page.getByTestId('player').filter({ hasText: 'Quinn' }).first().click();
	await expect(page.getByTestId('player-name')).toHaveText('Quinn');
	await page.getByRole('button', { name: 'Left the team' }).click();
	// Proposed: today; a role that ends today still counts today, so end it on its first day.
	await page.getByLabel('Last day in the team').fill('2026-10-12');
	await page.getByRole('button', { name: 'Left the team' }).click();
	await expect(page.getByTestId('roles')).toContainText('until Mon 12 Oct');
	await page.getByRole('button', { name: 'Undo "Left the team"' }).click();
	await expect(page.getByRole('button', { name: 'Left the team' })).toBeVisible();
	await expect(page.getByTestId('roles')).not.toContainText('until');
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([]);
});

test('"Left the team" proposes the start of a role that starts later', async ({ page }) => {
	await page.goto('/players');
	await page.getByRole('button', { name: 'Add player' }).click();
	const form = page.getByTestId('add-player');
	await form.getByLabel('First name').fill('Robin');
	await form.getByLabel('In the team from').fill('2026-11-02');
	await form.getByRole('button', { name: 'Save' }).click();
	await page.getByTestId('player').filter({ hasText: 'Robin' }).click();
	await page.getByRole('button', { name: 'Left the team' }).click();
	await expect(page.getByLabel('Last day in the team')).toHaveValue('2026-11-02');
});

test('a player that no longer exists gives a general message', async ({ page }) => {
	await page.goto('/players');
	await page.getByTestId('player').filter({ hasText: 'Robin' }).click();
	await page.route('**/api/players/*', (route) =>
		route.request().method() === 'PUT'
			? route.fulfill({ status: 404, json: { error: 'not_found' } })
			: route.fallback()
	);
	await page.getByRole('button', { name: 'Save name' }).click();
	await expect(page.getByRole('alert')).toHaveText('This no longer exists. Load the page again.');
	await page.unrouteAll({ behavior: 'ignoreErrors' });
});

test('set the training days and the reasons', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/settings');
	await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();
	await page.getByRole('button', { name: 'Add training day' }).click();
	const day = page.getByTestId('training-day');
	await day.getByLabel('Day').selectOption('2');
	await day.getByLabel('Start').fill('19:45');
	await day.getByLabel('End').fill('21:00');
	// Before the first save, the dates are the proposal for the chosen day.
	await expect(page.getByLabel('First training')).toHaveValue('2026-09-01');
	await expect(page.getByLabel('Last training')).toHaveValue('2027-05-25');
	const proposals: string[] = [];
	page.on('request', (request) => {
		if (request.url().includes('/api/training-settings/proposal')) proposals.push(request.url());
	});
	await page.getByRole('button', { name: 'Save training days' }).click();
	await expect(page.getByTestId('days-saved')).toHaveText('Saved.');
	// After the save, a change of days keeps the saved dates (the new day is not saved).
	await page.getByRole('button', { name: 'Add training day' }).click();
	await expect(page.getByTestId('training-day')).toHaveCount(2);
	await page.waitForTimeout(300);
	expect(proposals).toEqual([]);
	await expect(page.getByLabel('Last training')).toHaveValue('2027-05-25');

	await page.getByLabel('New reason').fill('Family');
	await page.getByRole('button', { name: 'Add reason' }).click();
	const family = page.getByTestId('reason').last();
	await expect(family.getByLabel('Name of the reason')).toHaveValue('Family');

	const holiday = page.locator('[data-reason="Holiday"]');
	await holiday.getByLabel('Hidden').check();
	await expect(holiday.getByLabel('Hidden')).toBeChecked();

	await page.reload();
	await expect(page.getByTestId('training-day').getByLabel('Start')).toHaveValue('19:45');
	await expect(page.getByLabel('First training')).toHaveValue('2026-09-01');
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([]);
});

test('players of last season come back with "Add all"', async ({ page }) => {
	// The server's clock has no last season, so the answer is replaced (story A, step 5); the API
	// tests cover the real carry.
	const errors = watchConsole(page);
	const lastSeason = [
		{ id: 9001, first_name: 'Rowan', last_name: null, role: 'plays_games', roles: [] },
		{ id: 9002, first_name: 'Sasha', last_name: 'Bos', role: 'only_trains', roles: [] }
	];
	const sent: unknown[] = [];
	await page.route('**/api/players/carry', async (route) => {
		sent.push(route.request().postDataJSON());
		await route.fulfill({ json: {} });
	});
	await page.route('**/api/players', async (route) => {
		if (route.request().method() !== 'GET') return route.fallback();
		const response = await route.fetch();
		const body = await response.json();
		// After "Add all", last season's players are gone from the list.
		await route.fulfill({ json: { ...body, last_season: sent.length ? [] : lastSeason } });
	});
	await page.goto('/players');
	const part = page.getByTestId('last-season');
	await expect(part.getByText('Rowan')).toBeVisible();
	await expect(part.getByText('Sasha Bos')).toBeVisible();
	await expect(part.getByRole('button', { name: 'Add Rowan to this season' })).toBeVisible();
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	await part.getByRole('button', { name: 'Add all' }).click();
	await expect(part).toHaveCount(0);
	expect(sent).toEqual([{ players: [9001, 9002] }]);
	expect(errors).toEqual([]);
	// A load of the list can still be under way when the test ends; its handler must not fail then.
	await page.unrouteAll({ behavior: 'ignoreErrors' });
});

test('a player of last season has no roles of this season yet', async ({ page }) => {
	// The server's clock has no last season, so the answer is replaced.
	const rowan = {
		id: 9001,
		first_name: 'Rowan',
		last_name: null,
		role: 'only_trains',
		roles: [{ id: 9101, role: 'only_trains', from: '2025-09-02', until: null }]
	};
	await page.route('**/api/players', async (route) => {
		if (route.request().method() !== 'GET') return route.fallback();
		const response = await route.fetch();
		await route.fulfill({ json: { ...(await response.json()), last_season: [rowan] } });
	});
	await page.goto('/players/9001');
	await expect(page.getByTestId('player-name')).toHaveText('Rowan');
	await expect(page.getByText('Not in the team this season.')).toBeVisible();
	await expect(page.getByRole('button', { name: 'Left the team' })).toHaveCount(0);
	await page.getByRole('button', { name: 'Add to the team' }).click();
	await expect(page.getByLabel('Only trains')).toBeChecked();
	await page.unrouteAll({ behavior: 'ignoreErrors' });
});
