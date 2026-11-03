// Who comes to a game (story A of attendance at games). The server's clock is Monday 12 October
// 2026 (DEV_NOW). These tests use the last coming game of the team, which no other test reads;
// the players come from the earlier tests.
import { expect, test, type Page } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

test.describe.configure({ mode: 'serial' });

type Game = { id: number; reason: string };

/** The last coming game of the team. */
async function lastGame(page: Page): Promise<string> {
	const answer = await (await page.request.get('/api/season')).json();
	const games: Game[] = answer.coming
		.filter((event: { game?: Game }) => event.game)
		.map((event: { game: Game }) => event.game)
		.filter((game: Game) => game.reason === 'team');
	return `/game/2026-2027/${games[games.length - 1].id}`;
}

const rows = (page: Page) => page.getByTestId('attendance-row');
const count = (page: Page) => page.getByTestId('attendance-count');
const titles = (page: Page) => page.getByTestId('attendance-group').getByRole('heading');

test('a tap saves, stays in its place and sorts after a reload; a second tap goes back', async ({
	page
}) => {
	const errors = watchConsole(page);
	await page.goto(await lastGame(page));
	await expect(titles(page)).toHaveText(['No answer']);
	const names = await rows(page).locator('span.truncate').allTextContents();
	const players = names.length;
	expect(players).toBeGreaterThan(1);
	await expect(count(page)).toHaveText(`${players} no answer`);

	// The second row: the rows do not move.
	const row = rows(page).nth(1);
	await row.getByRole('button', { name: /^Coming/ }).click();
	await expect(row.getByRole('button', { name: /^Coming/ })).toHaveAttribute(
		'aria-pressed',
		'true'
	);
	await expect(count(page)).toHaveText(`1 coming · ${players - 1} no answer`);
	await expect(rows(page).locator('span.truncate')).toHaveText(names);

	await page.reload();
	await expect(titles(page)).toHaveText(['No answer', 'Coming']);
	await expect(page.getByRole('list', { name: 'Coming' }).getByRole('listitem')).toHaveCount(1);

	// A tap on the chosen state goes back to "no answer".
	await rows(page)
		.last()
		.getByRole('button', { name: /^Coming/ })
		.click();
	await expect(count(page)).toHaveText(`${players} no answer`);
	await page.reload();
	await expect(titles(page)).toHaveText(['No answer']);
	expect(errors).toEqual([]);
});

test('"Reason" does not open by itself, and a tap on the chosen reason removes it', async ({
	page
}) => {
	await page.goto(await lastGame(page));
	const row = rows(page).first();
	await expect(row.getByRole('button', { name: /^Reason/ })).toHaveCount(0);
	await row.getByRole('button', { name: /^Not coming/ }).click();
	await expect(row.getByTestId('reasons')).toHaveCount(0);

	await row.getByRole('button', { name: 'Reason', exact: true }).click();
	const reason = row.getByTestId('reasons').getByRole('button').first();
	const name = await reason.textContent();
	await reason.click();
	await expect(row.getByRole('button', { name: `Reason: ${name?.trim()}` })).toBeVisible();

	// The reason stays when the state goes to "Coming" and back.
	await row.getByRole('button', { name: /^Coming/ }).click();
	await expect(count(page)).toContainText('1 coming');
	await row.getByRole('button', { name: /^Not coming/ }).click();
	await expect(row.getByRole('button', { name: /^Reason: / })).toBeVisible();

	// The reasons close at a tap on another row.
	await row.getByRole('button', { name: /^Reason/ }).click();
	await expect(row.getByTestId('reasons')).toBeVisible();
	await rows(page).nth(1).locator('span.truncate').click();
	await expect(row.getByTestId('reasons')).toHaveCount(0);

	await row.getByRole('button', { name: /^Reason/ }).click();
	await row.getByTestId('reasons').getByRole('button', { pressed: true }).click();
	await expect(row.getByRole('button', { name: 'Reason', exact: true })).toBeVisible();

	await row.getByRole('button', { name: /^Not coming/ }).click();
	await expect(row.getByRole('button', { name: /^Reason/ })).toHaveCount(0);
});

test('"Extra players" changes the count and stops at 0 and 10', async ({ page }) => {
	await page.goto(await lastGame(page));
	const less = page.getByRole('button', { name: 'One extra player less' });
	const more = page.getByRole('button', { name: 'One extra player more' });
	await expect(less).toBeDisabled();
	await more.click();
	await expect(count(page)).toContainText('1 coming');
	for (let number = 2; number <= 10; number += 1) await more.click();
	await expect(page.getByTestId('extra-number')).toHaveText('10');
	await expect(more).toBeDisabled();
	await expect(count(page)).toContainText('10 coming');
	await page.reload();
	await expect(page.getByTestId('extra-number')).toHaveText('10');
	for (let number = 9; number >= 0; number -= 1) await less.click();
	await expect(page.getByTestId('extra-number')).toHaveText('0');
	await expect(count(page)).not.toContainText('coming ·');
	await expect(less).toBeDisabled();
});

test('a tap that fails shows the stored state and "Not saved. Try again."', async ({ page }) => {
	await page.goto(await lastGame(page));
	await page.route('**/answers/**', (route) => route.abort());
	const row = rows(page).first();
	const coming = row.getByRole('button', { name: /^Coming/ });
	await coming.click();
	await expect(page.getByTestId('attendance-not-saved')).toHaveText('Not saved. Try again.');
	await expect(coming).toHaveAttribute('aria-pressed', 'false');
});

test('the count stays in view, the page fits a phone and is accessible', async ({ page }) => {
	await page.goto(await lastGame(page));
	await expect(count(page)).toBeVisible();
	await expectAccessible(page);
	// A low screen, so the rows must scroll: the count stays below the header of the page.
	await page.setViewportSize({ width: 390, height: 400 });
	await page.getByTestId('extra-players').evaluate((row) => row.scrollIntoView({ block: 'end' }));
	expect(await page.evaluate(() => window.scrollY)).toBeGreaterThan(200);
	const place = await count(page).boundingBox();
	expect(place?.y).toBeGreaterThanOrEqual(50);
	expect(place?.y).toBeLessThan(120);
	await expectNoHorizontalScroll(page, 320);
});

test('"Delete player" is not offered for a player with an answer', async ({ page }) => {
	await page.goto(await lastGame(page));
	const row = rows(page).first();
	const player = await row.getAttribute('data-player');
	await row.getByRole('button', { name: /^Coming/ }).click();
	await expect(count(page)).toContainText('1 coming');
	await page.goto(`/players/${player}`);
	await expect(page.getByRole('heading').first()).toBeVisible();
	await expect(page.getByRole('button', { name: 'Delete player' })).toHaveCount(0);
});

test('the row on "Season" shows the count', async ({ page }) => {
	const path = await lastGame(page);
	await page.goto('/season');
	const row = page.locator(`[data-testid="game"][data-id="${path.split('/').pop()}"]`);
	await expect(row.getByTestId('game-count')).toContainText('1 coming');
});

test('two taps from "Next" save an answer', async ({ page }) => {
	const path = await lastGame(page);
	const id = Number(path.split('/').pop());
	// A fixed answer of the start page: the last game of the team in "Next".
	const season = await (await page.request.get('/api/season')).json();
	const event = season.coming.find((event: { game?: Game }) => event.game?.id === id);
	await page.route('**/api/start', async (route) => {
		const response = await route.fetch();
		const start = await response.json();
		start.next = [
			{ ...event, game: { ...event.game, attendance: { coming: 6, not_coming: 0, no_answer: 2 } } }
		];
		await route.fulfill({ response, json: start });
	});
	await page.goto('/');
	const next = page.getByTestId('next');
	await expect(next.getByTestId('game-count')).toHaveText('6 coming · 2 no answer');
	await next.getByTestId('game-link').click();
	const before = await count(page).textContent();
	await rows(page)
		.first()
		.getByRole('button', { name: /^Not coming/ })
		.click();
	await expect(count(page)).not.toHaveText(before ?? '');
	const answer = await (await page.request.get(`/api/games/2026-2027/${id}`)).json();
	expect(
		answer.attendance.rows.some((row: { answer?: string }) => row.answer === 'not_coming')
	).toBe(true);
});
