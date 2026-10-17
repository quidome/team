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
	await form.getByRole('button', { name: 'Save' }).click();
	const player = page.getByTestId('player').filter({ hasText: 'Quinn' });
	await expect(player).toBeVisible();
	await expect(player).toContainText('Not in the team');
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
	await expect(page.getByTestId('player')).toHaveCount(2);
});

test('a player that no longer exists gives a general message', async ({ page }) => {
	await page.goto('/players');
	await page.getByTestId('player').first().click();
	await page.route('**/api/players/*', (route) =>
		route.request().method() === 'PUT'
			? route.fulfill({ status: 404, json: { error: 'not_found' } })
			: route.fallback()
	);
	await page.getByRole('button', { name: 'Save' }).click();
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

	// "Change": it was a Thursday. Only the day changes; the proposed window moves with it.
	await tuesday.getByRole('button', { name: 'Change' }).click();
	await form.getByLabel('Day').selectOption('4');
	await expect(form.getByLabel('First training')).toHaveValue('2026-09-03');
	await expect(form.getByLabel('Last training')).toHaveValue('2027-05-27');
	await form.getByRole('button', { name: 'Save weekly training' }).click();
	const thursday = page.getByTestId('weekly-training');
	await expect(thursday).toHaveText(/Thursday 19:45–21:00\s*· Thu 3 Sept – Thu 27 May/);

	// A typed window stays when the day changes.
	await thursday.getByRole('button', { name: 'Change' }).click();
	await form.getByLabel('First training').fill('2026-09-10');
	await form.getByLabel('Day').selectOption('2');
	await expect(form.getByLabel('First training')).toHaveValue('2026-09-10');
	await expect(form.getByLabel('Last training')).toHaveValue('2027-05-27');
	await form.getByRole('button', { name: 'Cancel' }).click();

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

test('the team page: add a player, change the role, remove from and add to the team', async ({
	page
}) => {
	const errors = watchConsole(page);
	// "In the team from" is proposed as 1 September of the season.
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

	// "Remove from the team" from tomorrow: still in the team today, on its last day.
	await page.getByRole('button', { name: 'Add player' }).click();
	await form.getByLabel('Player').selectOption({ label: 'New player' });
	await form.getByLabel('First name').fill('Tess');
	await form.getByRole('button', { name: 'Save' }).click();
	const tess = page.getByTestId('member').filter({ hasText: 'Tess' });
	await tess.getByRole('button', { name: /Tess/ }).click();
	await tess.getByRole('button', { name: 'Remove from the team' }).click();
	await expect(tess.getByLabel('Not in the team from')).toHaveValue('2026-10-12');
	await tess.getByRole('button', { name: 'Tomorrow' }).click();
	await expect(tess.getByLabel('Not in the team from')).toHaveValue('2026-10-13');
	await tess.getByRole('button', { name: 'Remove from the team' }).click();
	await expect(tess.getByTestId('last-day')).toHaveText('Last day today');

	// From today: under "Left the team" at once.
	await tess.getByRole('button', { name: /Tess/ }).click();
	await tess.getByRole('button', { name: 'Remove from the team' }).click();
	await tess.getByRole('button', { name: 'Remove from the team' }).click();
	const left = page.getByTestId('left-player').filter({ hasText: 'Tess' });
	await expect(left).toBeVisible();
	await expect(tess).toHaveCount(0);
	await left.getByRole('button', { name: /Tess/ }).click();
	await expect(left.getByTestId('member-roles')).toHaveText(
		'Plays games · Tue 1 Sept – Sun 11 Oct'
	);

	// "Add to the team": proposed today.
	await left.getByRole('button', { name: 'Add to the team' }).click();
	await expect(left.getByLabel('In the team from')).toHaveValue('2026-10-12');
	await left.getByRole('button', { name: 'Add to the team' }).click();
	await expect(tess).toBeVisible();
	await expect(left).toHaveCount(0);

	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([]);
});

test('"Remove from the team" asks first when it also removes a later role', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/team');
	await page.getByRole('button', { name: 'Add player' }).click();
	const form = page.getByTestId('add-player');
	await form.getByLabel('Player').selectOption({ label: 'New player' });
	await form.getByLabel('First name').fill('Vic');
	await form.getByLabel('Only trains').check();
	await form.getByRole('button', { name: 'Save' }).click();
	const vic = page.getByTestId('member').filter({ hasText: 'Vic' });

	// A planned "Change role" from Sun 1 Nov.
	await vic.getByRole('button', { name: /Vic/ }).click();
	await vic.getByRole('button', { name: 'Change role' }).click();
	await vic.getByLabel('From').fill('2026-11-01');
	await vic.getByRole('button', { name: 'Change role' }).click();
	await vic.getByRole('button', { name: /Vic/ }).click();
	await expect(vic.getByTestId('member-roles')).toHaveText(
		'Only trains · Tue 1 Sept – Sat 31 Oct, Plays games · from Sun 1 Nov'
	);

	// From today: the confirmation names the later role. Cancel keeps everything.
	await vic.getByRole('button', { name: 'Remove from the team' }).click();
	const messages: string[] = [];
	page.once('dialog', (dialog) => {
		messages.push(dialog.message());
		void dialog.dismiss();
	});
	await vic.getByRole('button', { name: 'Remove from the team' }).click();
	await expect.poll(() => messages).toEqual(['This also removes "Plays games · from Sun 1 Nov".']);
	await expect(vic.getByLabel('Not in the team from')).toBeVisible();

	page.once('dialog', (dialog) => void dialog.accept());
	await vic.getByRole('button', { name: 'Remove from the team' }).click();
	const left = page.getByTestId('left-player').filter({ hasText: 'Vic' });
	await expect(left).toBeVisible();
	await left.getByRole('button', { name: /Vic/ }).click();
	await expect(left.getByTestId('member-roles')).toHaveText(
		'Only trains · Tue 1 Sept – Sun 11 Oct'
	);
	await expectAccessible(page);

	// "Undo" puts both roles back.
	await left.getByRole('button', { name: 'Undo' }).click();
	await expect(left).toHaveCount(0);
	await vic.getByRole('button', { name: /Vic/ }).click();
	await expect(vic.getByTestId('member-roles')).toHaveText(
		'Only trains · Tue 1 Sept – Sat 31 Oct, Plays games · from Sun 1 Nov'
	);
	await expect(vic.getByRole('button', { name: 'Undo' })).toHaveCount(0);
	await expectNoHorizontalScroll(page, 320);
	// The two answers that ask for the confirmation.
	expect(errors).toEqual([
		'Failed to load resource: the server responded with a status of 409 (Conflict)',
		'Failed to load resource: the server responded with a status of 409 (Conflict)'
	]);
});

test('"Undo" for a player whose roles were all removed, and "Change role" to the same role', async ({
	page
}) => {
	const errors = watchConsole(page);
	await page.goto('/team');
	await page.getByRole('button', { name: 'Add player' }).click();
	const form = page.getByTestId('add-player');
	await form.getByLabel('Player').selectOption({ label: 'New player' });
	await form.getByLabel('First name').fill('Wren');
	await expect(form.getByLabel('In the team from')).toHaveValue('2026-09-01');
	await form.getByRole('button', { name: 'Today' }).click();
	await expect(form.getByLabel('In the team from')).toHaveValue('2026-10-12');
	await form.getByLabel('In the team from').fill('2026-11-02');
	await form.getByRole('button', { name: 'Save' }).click();
	const wren = page.getByTestId('member').filter({ hasText: 'Wren' });

	// "Change role" to the role Wren already has: refused.
	await wren.getByRole('button', { name: /Wren/ }).click();
	await wren.getByRole('button', { name: 'Change role' }).click();
	await wren.getByLabel('Plays games').check();
	await wren.getByLabel('From').fill('2026-11-09');
	await wren.getByRole('button', { name: 'Change role' }).click();
	await expect(wren.getByRole('alert')).toHaveText('The player already has this role.');
	await wren.getByRole('button', { name: 'Cancel' }).click();

	// From today: the role from Mon 2 Nov goes, so nothing is left in the season.
	await wren.getByRole('button', { name: 'Remove from the team' }).click();
	page.once('dialog', (dialog) => void dialog.accept());
	await wren.getByRole('button', { name: 'Remove from the team' }).click();
	const removed = page.getByTestId('removed').filter({ hasText: 'Wren' });
	await expect(removed).toHaveText(/Wren · not in the team this season\s*Undo/);
	await expect(wren).toHaveCount(0);
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);

	await removed.getByRole('button', { name: 'Undo' }).click();
	await expect(removed).toHaveCount(0);
	await wren.getByRole('button', { name: /Wren/ }).click();
	await expect(wren.getByTestId('member-roles')).toHaveText('Plays games · from Mon 2 Nov');
	expect(errors).toEqual([
		'Failed to load resource: the server responded with a status of 400 (Bad Request)',
		'Failed to load resource: the server responded with a status of 409 (Conflict)'
	]);
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
	const addAll = part.getByTestId('add-all');
	await expect(addAll.getByLabel('In the team from')).toHaveValue('2026-09-01');
	await addAll.getByRole('button', { name: 'Today' }).click();
	await expect(addAll.getByLabel('In the team from')).toHaveValue('2026-10-12');
	await expectNoHorizontalScroll(page, 320);
	await addAll.getByRole('button', { name: 'Add all' }).click();
	await expect(part).toHaveCount(0);
	expect(sent).toEqual([{ players: [9001, 9002], from: '2026-10-12' }]);
	expect(errors).toEqual([]);
	await page.unrouteAll({ behavior: 'ignoreErrors' });
});

test('a hidden player is listed only with "Show hidden players"', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/players');
	await page.getByRole('button', { name: 'Add player' }).click();
	await page.getByTestId('add-player').getByLabel('First name').fill('Uma');
	await page.getByTestId('add-player').getByRole('button', { name: 'Save' }).click();
	await page.getByTestId('player').filter({ hasText: 'Uma' }).click();
	await expect(page.getByTestId('player-name')).toHaveText('Uma');
	await expect(page.getByText('Never in the team.')).toBeVisible();
	await page.getByLabel('Hidden').check();
	await page.getByRole('button', { name: 'Save' }).click();
	await expect(page.getByLabel('Hidden')).toBeChecked();

	await page.goto('/players');
	const uma = page.getByTestId('player').filter({ hasText: 'Uma' });
	await expect(page.getByTestId('player').first()).toBeVisible();
	await expect(uma).toHaveCount(0);
	await page.getByLabel('Show hidden players').check();
	await expect(uma).toContainText('hidden');

	// Delete, after a confirmation.
	await uma.click();
	page.once('dialog', (dialog) => void dialog.accept());
	await page.getByRole('button', { name: 'Delete player' }).click();
	await expect(page.getByRole('heading', { name: 'Players' })).toBeVisible();
	await page.getByLabel('Show hidden players').check();
	await expect(page.getByTestId('player').first()).toBeVisible();
	await expect(uma).toHaveCount(0);
	expect(errors).toEqual([]);
});

test('"Correct" in the history changes the start of a role', async ({ page }) => {
	// Sky has the roles of the team page test: Only trains from Tue 1 Sep until Sun 11 Oct, then
	// Plays games.
	const errors = watchConsole(page);
	await page.goto('/players');
	await page.getByTestId('player').filter({ hasText: 'Sky' }).first().click();
	const roles = page.getByTestId('history-role');
	await expect(roles).toHaveText([
		/Plays games · from Mon 12 Oct\s*Correct/,
		/Only trains · Tue 1 Sept – Sun 11 Oct\s*Correct/
	]);
	await roles.nth(1).getByRole('button', { name: 'Correct' }).click();
	const form = page.getByTestId('correct-form');
	await expect(form.getByLabel('From')).toHaveValue('2026-09-01');
	await expect(form.getByLabel('From')).toHaveAttribute('min', '2026-07-01');
	await expect(form.getByLabel(/^Until/)).toHaveValue('2026-10-11');
	await form.getByLabel('From').fill('2026-09-08');
	await form.getByRole('button', { name: 'Save' }).click();
	await expect(roles.nth(1)).toContainText('Only trains · Tue 8 Sept – Sun 11 Oct');

	// An overlap is refused.
	await roles.nth(1).getByRole('button', { name: 'Correct' }).click();
	await form.getByLabel(/^Until/).fill('2026-10-20');
	await form.getByRole('button', { name: 'Save' }).click();
	await expect(form.getByRole('alert')).toHaveText('This player already has a role on that date.');
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([
		'Failed to load resource: the server responded with a status of 400 (Bad Request)'
	]);
});
