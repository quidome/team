// Players and settings (attendance at training, story A). The server's clock is 12 October 2026
// (DEV_NOW), so the season is 2026-2027. The tests share one database and run in this order.
import { expect, test } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

test.describe.configure({ mode: 'serial' });

test('the start page links to the team, players and settings', async ({ page }) => {
	await page.goto('/');
	const links = page.getByTestId('team-links');
	await expect(links.getByRole('link', { name: 'Team' })).toBeVisible();
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

test('set the weekly trainings and the reasons', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/settings');
	await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();
	await expect(page.getByText('No weekly trainings yet.')).toBeVisible();

	// Add: the window is the proposal for the chosen day.
	await page.getByRole('button', { name: 'Add weekly training' }).click();
	const form = page.getByTestId('weekly-form');
	await form.getByLabel('Day').selectOption('2');
	await expect(form.getByLabel('First training')).toHaveValue('2026-09-01');
	await expect(form.getByLabel('Last training')).toHaveValue('2027-05-25');
	await expect(form.getByLabel('First training')).toHaveAttribute('min', '2026-07-01');
	await expect(form.getByLabel('Last training')).toHaveAttribute('max', '2027-06-30');
	await form.getByLabel('Start').fill('19:45');
	await form.getByLabel('End').fill('21:00');
	await form.getByRole('button', { name: 'Save weekly training' }).click();
	const tuesday = page.getByTestId('weekly-training');
	await expect(tuesday).toHaveText(/Tuesday 19:45–21:00\s*· Tue 1 Sept – Tue 25 May/);

	// "Change": it was a Thursday.
	await tuesday.getByRole('button', { name: 'Change' }).click();
	await form.getByLabel('Day').selectOption('4');
	await form.getByLabel('First training').fill('2026-09-03');
	await form.getByLabel('Last training').fill('2027-05-27');
	await form.getByRole('button', { name: 'Save weekly training' }).click();
	const thursday = page.getByTestId('weekly-training');
	await expect(thursday).toHaveText(/Thursday 19:45–21:00\s*· Thu 3 Sept – Thu 27 May/);

	// A second Thursday in the same dates is refused.
	await page.getByRole('button', { name: 'Add weekly training' }).click();
	await form.getByLabel('Day').selectOption('4');
	await expect(form.getByLabel('First training')).toHaveValue('2026-09-03');
	await form.getByLabel('Start').fill('20:00');
	await form.getByLabel('End').fill('21:30');
	await form.getByRole('button', { name: 'Save weekly training' }).click();
	await expect(page.getByRole('alert')).toHaveText(
		'There is already a weekly training on Thursday in these dates.'
	);
	await form.getByRole('button', { name: 'Cancel' }).click();
	await expect(page.getByTestId('weekly-training')).toHaveCount(1);
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);

	// Delete, after a confirmation.
	page.once('dialog', (dialog) => void dialog.accept());
	await thursday.getByRole('button', { name: 'Delete' }).click();
	await expect(page.getByText('No weekly trainings yet.')).toBeVisible();

	await page.getByLabel('New reason').fill('Family');
	await page.getByRole('button', { name: 'Add reason' }).click();
	const family = page.getByTestId('reason').last();
	await expect(family.getByLabel('Name of the reason')).toHaveValue('Family');

	const holiday = page.locator('[data-reason="Holiday"]');
	await holiday.getByLabel('Hidden').check();
	await expect(holiday.getByLabel('Hidden')).toBeChecked();

	await page.reload();
	await expect(page.getByText('No weekly trainings yet.')).toBeVisible();
	await expect(page.locator('[data-reason="Holiday"]').getByLabel('Hidden')).toBeChecked();
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	// Only the refused second Thursday.
	expect(errors).toEqual([
		'Failed to load resource: the server responded with a status of 400 (Bad Request)'
	]);
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
		// A proposed start that is not today, as in summer before the first training date.
		await route.fulfill({
			json: { ...(await response.json()), last_season: [rowan], proposed_from: '2026-11-03' }
		});
	});
	await page.goto('/players/9001');
	await expect(page.getByTestId('player-name')).toHaveText('Rowan');
	await expect(page.getByText('Not in the team this season.')).toBeVisible();
	await expect(page.getByRole('button', { name: 'Left the team' })).toHaveCount(0);
	await page.getByRole('button', { name: 'Add to the team' }).click();
	await expect(page.getByLabel('Only trains')).toBeChecked();
	await expect(page.getByLabel('In the team from')).toHaveValue('2026-11-03');
	await page.unrouteAll({ behavior: 'ignoreErrors' });
});

test('the team page: add a player, change the role, remove from and add to the team', async ({
	page
}) => {
	const errors = watchConsole(page);
	// A Tuesday weekly training: the proposed start of a role is its first date, Tue 1 Sep.
	await page.goto('/settings');
	await page.getByRole('button', { name: 'Add weekly training' }).click();
	const weekly = page.getByTestId('weekly-form');
	await weekly.getByLabel('Day').selectOption('2');
	await expect(weekly.getByLabel('First training')).toHaveValue('2026-09-01');
	await weekly.getByLabel('Start').fill('19:45');
	await weekly.getByLabel('End').fill('21:00');
	await weekly.getByRole('button', { name: 'Save weekly training' }).click();
	await expect(page.getByTestId('weekly-training')).toHaveCount(1);

	await page.goto('/team');
	await page.getByRole('button', { name: 'Add player' }).click();
	const form = page.getByTestId('add-player');
	await form.getByLabel('Player').selectOption({ label: 'New player' });
	await form.getByLabel('First name').fill('Sky');
	await form.getByLabel('Only trains').check();
	await expect(form.getByLabel('In the team from')).toHaveValue('2026-09-01');
	await expect(form.getByLabel('In the team from')).toHaveAttribute('min', '2026-07-01');
	await expect(form.getByLabel('In the team from')).toHaveAttribute('max', '2027-06-30');
	await form.getByRole('button', { name: 'Save' }).click();
	const sky = page.getByTestId('member').filter({ hasText: 'Sky' });
	await expect(sky).toContainText('Only trains');

	// The same name again: allowed, with a warning.
	await page.getByRole('button', { name: 'Add player' }).click();
	await form.getByLabel('Player').selectOption({ label: 'New player' });
	await form.getByLabel('First name').fill('sky');
	await form.getByRole('button', { name: 'Save' }).click();
	await expect(page.getByTestId('same-name')).toHaveText(
		'There is already a player with this name.'
	);

	// "Change role" from today: the role until yesterday, the new one from today.
	await sky.first().getByRole('button', { name: /Sky/ }).click();
	await sky.first().getByRole('button', { name: 'Change role' }).click();
	await expect(sky.first().getByLabel('Plays games')).toBeChecked();
	await expect(sky.first().getByLabel('From')).toHaveValue('2026-10-12');
	await sky.first().getByRole('button', { name: 'Change role' }).click();
	await expect(sky.first()).toContainText('Plays games');
	await sky.first().getByRole('button', { name: /Sky/ }).click();
	await expect(sky.first().getByTestId('member-roles')).toHaveText(
		'Only trains · Tue 1 Sept – Sun 11 Oct, Plays games · from Mon 12 Oct'
	);

	// "Remove from the team" on Thu 1 Oct: under "Left the team".
	await page.getByRole('button', { name: 'Add player' }).click();
	await form.getByLabel('Player').selectOption({ label: 'New player' });
	await form.getByLabel('First name').fill('Tess');
	await form.getByRole('button', { name: 'Save' }).click();
	const tess = page.getByTestId('member').filter({ hasText: 'Tess' });
	await tess.getByRole('button', { name: /Tess/ }).click();
	await tess.getByRole('button', { name: 'Remove from the team' }).click();
	await expect(tess.getByLabel('Last day in the team')).toHaveValue('2026-10-12');
	await tess.getByLabel('Last day in the team').fill('2026-10-01');
	await tess.getByRole('button', { name: 'Remove from the team' }).click();
	const left = page.getByTestId('left-player').filter({ hasText: 'Tess' });
	await expect(left).toBeVisible();
	await expect(tess).toHaveCount(0);

	// "Add to the team": proposed the day after the end, or today when later.
	await left.getByRole('button', { name: /Tess/ }).click();
	await left.getByRole('button', { name: 'Add to the team' }).click();
	await expect(left.getByLabel('In the team from')).toHaveValue('2026-10-12');
	await left.getByRole('button', { name: 'Add to the team' }).click();
	await expect(tess).toBeVisible();
	await expect(left).toHaveCount(0);

	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([]);
});

test('the team page shows the chosen team, or that it is not chosen yet', async ({ page }) => {
	// The answer is replaced: which team is chosen depends on other test files.
	let teamName: string | null = 'U14-1';
	await page.route('**/api/team', async (route) => {
		const response = await route.fetch();
		await route.fulfill({ json: { ...(await response.json()), team_name: teamName } });
	});
	await page.goto('/team');
	await expect(page.getByTestId('team-heading')).toHaveText('U14-1 · 2026-2027');
	teamName = null;
	await page.reload();
	await expect(page.getByTestId('team-heading')).toHaveText('Team not chosen yet');
	await expect(page.getByRole('link', { name: 'Choose the team' })).toHaveAttribute(
		'href',
		'/choose'
	);
	await page.unrouteAll({ behavior: 'ignoreErrors' });
});

test('the team page brings last season\'s players back with "Add all"', async ({ page }) => {
	// The server's clock has no last season, so the answers are replaced; the API tests cover the
	// real "Add all".
	const errors = watchConsole(page);
	const lastSeason = [
		{ id: 9001, first_name: 'Rowan', last_name: null, role: 'plays_games', roles: [] },
		{ id: 9002, first_name: 'Sasha', last_name: 'Bos', role: 'only_trains', roles: [] }
	];
	const sent: unknown[] = [];
	await page.route('**/api/team', async (route) => {
		const response = await route.fetch();
		await route.fulfill({ json: { ...(await response.json()), last_season: lastSeason } });
	});
	await page.route('**/api/players/carry', async (route) => {
		sent.push(route.request().postDataJSON());
		const team = await (await page.request.get('/api/team')).json();
		await route.fulfill({ json: { ...team, last_season: [] } });
	});
	await page.goto('/team');
	const part = page.getByTestId('last-season');
	await expect(part.getByText('Rowan')).toBeVisible();
	await expect(part.getByText('Sasha Bos')).toBeVisible();
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	await part.getByRole('button', { name: 'Add all' }).click();
	await expect(part).toHaveCount(0);
	expect(sent).toEqual([{ players: [9001, 9002] }]);
	expect(errors).toEqual([]);
	await page.unrouteAll({ behavior: 'ignoreErrors' });
});
