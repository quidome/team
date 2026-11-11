// "Not played" (story E of attendance at games). The server's clock is Monday 12 October 2026
// (DEV_NOW). These tests mark games that other tests read, so they run last (project
// `not-played`), and each test undoes its mark.
import { expect, test } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

test.describe.configure({ mode: 'serial' });

/** The away game of the team on Saturday 7 November 2026 (coming). */
const AWAY_7_NOV = 1073159;
/** The away game of the team on Saturday 10 October 2026 (past). */
const AWAY_10_OCT = 1073147;

const gamePage = (id: number) => `/game/2026-2027/${id}`;
const apiPath = (id: number, part: string) => `/api/games/2026-2027/${id}/${part}`;
const origin = { headers: { Origin: 'http://localhost:4100' } };

test('"Mark as not played" closes the list, keeps it changeable, and "Undo" brings everything back', async ({
	page
}) => {
	const errors = watchConsole(page);
	await page.goto(gamePage(AWAY_7_NOV));
	await expect(page.getByTestId('attendance-row').first()).toBeVisible();
	// A coming away game has nothing to mark yet: no panel.
	await expect(page.getByTestId('administration')).toHaveCount(0);
	await page.getByRole('button', { name: 'Mark as not played' }).click();

	const mark = page.getByTestId('mark-not-played');
	await expect(mark).toContainText('Not played');
	await expect(page.getByTestId('attendance-row')).toHaveCount(0);
	await expect(page.getByRole('button', { name: 'Mark as not played' })).toHaveCount(0);
	// Before the start time there is no switch.
	await expect(page.getByTestId('mark-drives')).toHaveCount(0);

	// The list opens behind a tap, and an answer can still be changed.
	await page.getByTestId('show-list').click();
	const row = page.getByTestId('attendance-row').first();
	await row.getByRole('button', { name: /^Coming/ }).click();
	await expect(row.getByRole('button', { name: /^Coming/ })).toHaveAttribute(
		'aria-pressed',
		'true'
	);
	await row.getByRole('button', { name: /^Coming/ }).click();

	// The row on "Season" shows "· Not played" and no count.
	await page.goto('/season');
	const seasonRow = page.locator(`[data-testid="game"][data-id="${AWAY_7_NOV}"]`);
	await expect(seasonRow.getByTestId('game-not-played')).toHaveText('· Not played');
	await expect(seasonRow.getByTestId('game-count')).toHaveCount(0);

	await page.goto(gamePage(AWAY_7_NOV));
	await expect(page.getByTestId('show-list')).toBeVisible();
	await mark.getByRole('button', { name: 'Undo' }).click();
	await expect(page.getByTestId('mark-not-played')).toHaveCount(0);
	await expect(page.getByTestId('attendance-row').first()).toBeVisible();
	await expect(page.getByRole('button', { name: 'Mark as not played' })).toBeVisible();
	expect(errors).toEqual([]);
});

test('a game marked "Not played" says so on the start page', async ({ page }) => {
	await page.request.put(apiPath(AWAY_10_OCT, 'not-played'), origin);
	try {
		// The game as the server gives it now, shown as a coming game of the team (the clock has no
		// game of the team in the two weeks of the list).
		const season = await (await page.request.get('/api/season')).json();
		const event = season.past.find((e: { game?: { id: number } }) => e.game?.id === AWAY_10_OCT);
		await page.route('**/api/start', async (route) => {
			const start = await (await route.fetch()).json();
			start.next = [{ event, look: 'lighter', side: 'away', opponent: 'De Dunckers M16 3' }];
			await route.fulfill({ json: start });
		});
		await page.goto('/');
		const row = page.getByTestId('next').locator(`[data-testid="game"][data-id="${AWAY_10_OCT}"]`);
		await expect(row).toHaveAttribute('data-look', 'lighter');
		await expect(row.getByTestId('game-title')).toHaveText('Away game at De Dunckers M16 3');
		await expect(row.getByTestId('game-not-played')).toHaveText('Not played');
	} finally {
		await page.request.delete(apiPath(AWAY_10_OCT, 'not-played'), origin);
	}
});

test('after the start time a not played away game has the row "The drives count"', async ({
	page
}) => {
	await page.request.put(apiPath(AWAY_10_OCT, 'drivers'), { ...origin, data: { drivers: 1 } });
	await page.goto(gamePage(AWAY_10_OCT));
	await page.getByRole('button', { name: 'Mark as not played' }).click();
	const drove = page.getByTestId('mark-drives');
	await expect(drove).toHaveAttribute('data-done', 'false');
	// Optional, so never orange.
	await expect(drove).toHaveAttribute('data-next', 'false');
	await expect(drove).toContainText('No drive counts');
	await drove.getByRole('button', { name: 'Count the drives' }).click();
	await expect(drove).toHaveAttribute('data-done', 'true');
	await expect(drove).toContainText('The drives count');
	// The rows of the panel, in their order.
	const rows = await page
		.getByTestId('administration')
		.locator('[data-done]')
		.evaluateAll((elements) => elements.map((element) => element.getAttribute('data-testid')));
	expect(rows).toEqual(['mark-not-played', 'mark-drives', 'mark-checked']);
	await page.reload();
	await expect(drove).toHaveAttribute('data-done', 'true');

	await drove.getByRole('button', { name: 'Undo' }).click();
	await expect(drove).toHaveAttribute('data-done', 'false');
	await page.getByTestId('mark-not-played').getByRole('button', { name: 'Undo' }).click();
	await expect(page.getByTestId('mark-drives')).toHaveCount(0);
});

test('a not played game shows no note of a driver who is not coming', async ({ page }) => {
	const game = await (await page.request.get(`/api/games/2026-2027/${AWAY_7_NOV}`)).json();
	const player = game.players[0].id;
	await page.request.put(apiPath(AWAY_7_NOV, 'drivers'), { ...origin, data: { drivers: 1 } });
	await page.request.put(apiPath(AWAY_7_NOV, 'assignments/drive/1'), {
		...origin,
		data: { player }
	});
	await page.request.put(apiPath(AWAY_7_NOV, `answers/${player}`), {
		...origin,
		data: { answer: 'not_coming' }
	});
	try {
		await page.goto(gamePage(AWAY_7_NOV));
		const place = page.getByTestId('place-player').first();
		await expect(place).toContainText('· not coming');
		await page.getByRole('button', { name: 'Mark as not played' }).click();
		await expect(page.getByTestId('mark-not-played')).toBeVisible();
		await expect(place).not.toContainText('not coming');
		await page.getByTestId('show-list').click();
		await expect(page.getByTestId('attendance-row').first()).toBeVisible();
		await expect(place).not.toContainText('not coming');
		await page.reload();
		await expect(place).not.toContainText('not coming');
	} finally {
		await page.request.delete(apiPath(AWAY_7_NOV, 'not-played'), origin);
		await page.request.delete(apiPath(AWAY_7_NOV, `answers/${player}`), origin);
		await page.request.delete(apiPath(AWAY_7_NOV, 'assignments/drive/1'), origin);
	}
});

test('a tap that fails shows "Not saved. Try again." and the stored state', async ({ page }) => {
	await page.goto(gamePage(AWAY_7_NOV));
	await page.route('**/not-played', (route) => route.abort());
	await page.getByRole('button', { name: 'Mark as not played' }).click();
	await expect(page.getByTestId('not-saved')).toHaveText('Not saved. Try again.');
	await expect(page.getByTestId('mark-not-played')).toHaveCount(0);
	await expect(page.getByRole('button', { name: 'Mark as not played' })).toBeVisible();
});

test('a not played game fits a phone and is accessible', async ({ page }) => {
	await page.request.put(apiPath(AWAY_10_OCT, 'not-played'), origin);
	try {
		await page.setViewportSize({ width: 320, height: 700 });
		await page.goto(gamePage(AWAY_10_OCT));
		await expect(page.getByTestId('mark-not-played')).toBeVisible();
		await expect(page.getByTestId('mark-drives')).toBeVisible();
		await expectNoHorizontalScroll(page, 320);
		await expectAccessible(page);
	} finally {
		await page.request.delete(apiPath(AWAY_10_OCT, 'not-played'), origin);
	}
});
