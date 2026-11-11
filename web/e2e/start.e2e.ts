// The start page as a landing page (story B of start page and season). The server's clock is
// Monday 12 October 2026 (DEV_NOW). These tests run after the tests of "Season" (one database, in
// this order): Blue Drakes M16 1 is chosen, and there are Tuesday and Thursday trainings.
import { expect, test } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

test.describe.configure({ mode: 'serial' });

/** Replaces the open items of the answer of the start page. */
async function withOpenItems(page: import('@playwright/test').Page, items: object[]) {
	await page.route('**/api/start', async (route) => {
		const start = await (await route.fetch()).json();
		start.open_items = items;
		await route.fulfill({ json: start });
	});
}

test('the start page shows the team and the date, the open items, the events and the links', async ({
	page
}) => {
	const errors = watchConsole(page);
	await page.goto('/');
	await expect(page.getByTestId('chosen-team')).toHaveText('Blue Drakes M16 1');
	await expect(page.getByTestId('today')).toHaveText('Monday 12 October');
	// In October the header has no season, and the team is changed on the settings.
	await expect(page.getByTestId('season')).toHaveCount(0);
	await expect(page.getByRole('link', { name: 'Change', exact: true })).toHaveCount(0);
	const parts = page.locator(
		'[data-testid="open-items"], [data-testid="next"], [data-testid="team-links"]'
	);
	await expect(parts).toHaveCount(3);
	const order = await parts.evaluateAll((elements) =>
		elements.map((element) => element.getAttribute('data-testid'))
	);
	expect(order).toEqual(['open-items', 'next', 'team-links']);
	await expect(page.getByTestId('team-links').getByRole('link')).toHaveText([
		'Season',
		'Attendance',
		'Duties',
		'Report',
		'More'
	]);
	// The old parts are gone.
	await expect(page.getByTestId('game-list-card')).toHaveCount(0);
	await expect(page.getByTestId('training-card')).toHaveCount(0);
	await expect(page.getByTestId('unseen-changes')).toHaveCount(0);
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([]);
});

test('without open items the start page says "Nothing needs you."', async ({ page }) => {
	await withOpenItems(page, []);
	await page.goto('/');
	await expect(page.getByTestId('nothing-open')).toHaveText('Nothing needs you.');
	await expect(page.getByTestId('open-item')).toHaveCount(0);
});

test('the open items of the game list show their words', async ({ page }) => {
	await withOpenItems(page, [
		{ kind: 'reading_held_back' },
		{ kind: 'out_of_date', since: '2026-10-09' }
	]);
	await page.goto('/');
	await expect(page.getByTestId('open-item')).toHaveText([
		'The last reading of the game list was held back.',
		'The games may be out of date since Fri 9 Oct.'
	]);
	await expect(page.locator('[data-kind="reading_held_back"]')).toHaveAttribute(
		'href',
		'/season#game-list'
	);
});

/** A game of an open item: the first coming game of the team, so its page opens. */
async function teamGame(page: import('@playwright/test').Page) {
	const season = await (await page.request.get('/api/season')).json();
	const game = season.coming
		.map((event: { game?: { id: number; reason: string } }) => event.game)
		.find((game?: { reason: string }) => game?.reason === 'team');
	return {
		season: '2026-2027',
		id: game.id as number,
		date: '2026-10-31',
		start_time: '16:00',
		home: 'Blue Drakes M16 1',
		away: 'BC Utrecht Cangeroes M16 2'
	};
}

test('the warnings of one game show their words, and a tap opens the game', async ({ page }) => {
	const errors = watchConsole(page);
	const game = await teamGame(page);
	await withOpenItems(page, [
		{ kind: 'too_few_players', games: [{ ...game, coming: 5, no_answer: 3, enough: 7 }] },
		{ kind: 'driver_not_coming', games: [{ ...game, drivers: ['Anna'] }] },
		{ kind: 'answers_moved', games: [{ ...game, date: '2026-11-04' }] }
	]);
	await page.goto('/');
	await expect(page.getByTestId('open-item')).toHaveText([
		'Game Sat 31 Oct: 5 of 7 players, 3 no answer',
		'Away game Sat 31 Oct: driver Anna is not coming',
		'Game Wed 4 Nov moved: check the answers'
	]);
	await page.locator('[data-kind="too_few_players"]').click();
	await expect(page).toHaveURL(new RegExp(`/game/2026-2027/${game.id}\\?from=start$`));
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/$/);
	expect(errors).toEqual([]);
});

test('the warnings of more games or drivers have their own words, and a tap lists the games', async ({
	page
}) => {
	const game = await teamGame(page);
	const other = { ...game, id: game.id + 1, date: '2026-11-07' };
	await withOpenItems(page, [
		{
			kind: 'too_few_players',
			games: [
				{ ...game, coming: 6, no_answer: 0, enough: 7 },
				{ ...other, coming: 4, no_answer: 2, enough: 7 }
			]
		},
		{ kind: 'driver_not_coming', games: [{ ...game, drivers: ['Anna', 'Bram'] }] },
		{ kind: 'answers_moved', games: [game, other] }
	]);
	await page.goto('/');
	await expect(page.getByTestId('open-item')).toHaveText([
		'2 games: too few players',
		'Away game Sat 31 Oct: 2 drivers are not coming',
		'2 games moved: check the answers'
	]);
	await page.locator('[data-kind="too_few_players"]').click();
	await expect(
		page.locator('#open-item-too_few_players').getByTestId('open-item-game')
	).toHaveCount(2);
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
});

test('one game without "no answer", and more games with a driver not coming', async ({ page }) => {
	const game = await teamGame(page);
	const other = { ...game, id: game.id + 1, date: '2026-11-07' };
	await withOpenItems(page, [
		{ kind: 'too_few_players', games: [{ ...game, coming: 5, no_answer: 0, enough: 7 }] },
		{
			kind: 'driver_not_coming',
			games: [
				{ ...game, drivers: ['Anna'] },
				{ ...other, drivers: ['Bram'] }
			]
		}
	]);
	await page.goto('/');
	await expect(page.getByTestId('open-item')).toHaveText([
		'Game Sat 31 Oct: 5 of 7 players',
		'2 games: a driver is not coming'
	]);
});

test('a training day not opened is an open item that opens it', async ({ page }) => {
	await page.goto('/');
	const item = page.locator('[data-testid="open-item"][data-kind="not_opened"]');
	await expect(item).toHaveText(
		/^(Training day \w{3} \d{1,2} \w{3} not opened|\d+ training days not opened)$/
	);
	await item.click();
	await expect(page).toHaveURL(/\/training\/\d{4}-\d{2}-\d{2}\?from=start$/);
	await expect(page.getByRole('button', { name: 'Open training' })).toBeVisible();
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/$/);
});

test('a training not checked is an open item that opens it', async ({ page }) => {
	await withOpenItems(page, [{ kind: 'not_checked', first: '2026-10-06', count: 1 }]);
	await page.goto('/');
	const item = page.locator('[data-testid="open-item"][data-kind="not_checked"]');
	await expect(item).toHaveText('Training Tue 6 Oct not checked');
	await item.click();
	await expect(page).toHaveURL(/\/training\/2026-10-06\?from=start$/);
});

test('a past training or duty game is not in the list of events, only as an open item', async ({
	page
}) => {
	// A past home game of another club team with a place of the team: a duty game not checked.
	const season = await (await page.request.get('/api/season')).json();
	const duty = season.past
		.map((event: { game?: { id: number; reason: string } }) => event.game)
		.find((game?: { reason: string }) => game?.reason === 'duty');
	const places = `/api/games/2026-2027/${duty.id}/places/table/2`;
	const origin = { headers: { Origin: 'http://localhost:4100' } };
	await page.request.put(places, { ...origin, data: { team: 184 } });
	try {
		await page.goto('/');
		await expect(page.getByTestId('next').getByTestId('training-row').first()).toBeVisible();
		const dates = await page
			.getByTestId('next')
			.getByTestId('training-row')
			.evaluateAll((rows) => rows.map((row) => row.getAttribute('data-date') ?? ''));
		expect(dates.length).toBeGreaterThan(0);
		expect(dates.every((date) => date >= '2026-10-12')).toBe(true);
		await expect(
			page.getByTestId('next').locator(`[data-testid="game"][data-id="${duty.id}"]`)
		).toHaveCount(0);
		await expect(
			page.locator('[data-testid="open-item"][data-kind="duty_games_not_checked"]')
		).toBeVisible();
	} finally {
		await page.request.put(places, { ...origin, data: { team: null } });
	}
});

test('"Next" shows the trainings of two weeks, and a tap opens one', async ({ page }) => {
	await page.goto('/');
	const next = page.getByTestId('next');
	await expect(next.getByTestId('day').first()).toHaveText('Tue 13 Oct');
	// No possible duty games in "Next".
	await expect(next.locator('[data-testid="game"][data-of-team="false"]')).toHaveCount(0);
	await next.getByTestId('training-row').first().getByRole('button').click();
	await expect(page).toHaveURL(/\/training\/2026-10-13\?from=start$/);
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/$/);
});

test('the duty games not checked have their own words', async ({ page }) => {
	const game = (id: number, date: string) => ({
		season: '2026-2027',
		id,
		date,
		start_time: '13:00',
		home: 'Blue Drakes X12 1',
		away: 'New Stars X12 2'
	});
	await withOpenItems(page, [
		{ kind: 'duty_games_not_checked', games: [game(1081913, '2026-10-03')] }
	]);
	await page.goto('/');
	const item = page.locator('[data-testid="open-item"][data-kind="duty_games_not_checked"]');
	await expect(item).toHaveText('Duty game Sat 3 Oct not checked');
	await expect(item).toHaveAttribute('href', '/game/2026-2027/1081913?from=start');

	await page.unrouteAll({ behavior: 'ignoreErrors' });
	await withOpenItems(page, [
		{
			kind: 'duty_games_not_checked',
			games: [game(1081913, '2026-10-03'), game(1081923, '2026-10-10')]
		}
	]);
	await page.goto('/');
	await expect(item).toHaveText('2 duty games not checked');
	await item.click();
	await expect(
		page.locator('#open-item-duty_games_not_checked').getByTestId('open-item-game')
	).toHaveText([
		'Sat 3 Oct · 13:00 Blue Drakes X12 1 – New Stars X12 2',
		'Sat 10 Oct · 13:00 Blue Drakes X12 1 – New Stars X12 2'
	]);
});

test('the past games of the team that are not checked open their list, and a tap opens one', async ({
	page
}) => {
	await page.goto('/');
	// From the start of the season: the past games of the test data are not checked.
	const item = page.locator('[data-testid="open-item"][data-kind="games_not_checked"]');
	await expect(item).toHaveText(/^\d+ games not checked$/);
	await expect(item).toHaveAttribute('aria-expanded', 'false');
	await item.click();
	await expect(item).toHaveAttribute('aria-expanded', 'true');
	const games = page.locator('#open-item-games_not_checked').getByTestId('open-item-game');
	await expect(games.first()).toHaveText(/^\w{3} \d{1,2} \w{3,4} · /);
	const count = Number((await item.textContent())?.match(/\d+/)?.[0]);
	await expect(games).toHaveCount(count);
	await games.first().click();
	await expect(page).toHaveURL(/\/game\/2026-2027\/\d+\?from=start$/);
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/$/);
});

test('a start page that cannot be loaded says so', async ({ page }) => {
	await page.route('**/api/start', (route) => route.fulfill({ status: 500, json: {} }));
	await page.goto('/');
	await expect(page.getByTestId('start-error')).toHaveText(
		'The start page could not be loaded. Load the page again.'
	);
});

test('screenshots of the start page for the look', async ({ page }) => {
	for (const colorScheme of ['light', 'dark'] as const) {
		await page.emulateMedia({ colorScheme });
		await page.goto('/');
		await expect(page.getByTestId('next')).toBeVisible();
		await page.screenshot({ path: `test-results/look/landing-${colorScheme}.png`, fullPage: true });
	}
});
