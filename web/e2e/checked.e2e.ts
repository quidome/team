// "Checked" on the page of a game (story C of attendance at games). The server's clock is Monday
// 12 October 2026 (DEV_NOW). Every past game of the team of the season that is not checked is an
// open item. These tests check a past game that other tests read, so they run last (project
// `checked`), and each test undoes its check.
import { expect, test } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

test.describe.configure({ mode: 'serial' });

/** The away game of the team on Saturday 10 October 2026 (past). */
const AWAY_10_OCT = 1073147;
const gamePage = `/game/2026-2027/${AWAY_10_OCT}`;
const checkedApi = `/api/games/2026-2027/${AWAY_10_OCT}/checked`;
const origin = { headers: { Origin: 'http://localhost:4100' } };

test('a past game that is not checked shows its list open, with "There" and "Absent"', async ({
	page
}) => {
	await page.goto(gamePage);
	// No release day any more: the list of a past game is open until it is checked.
	await expect(page.getByTestId('closed-list')).toHaveCount(0);
	const row = page.getByTestId('attendance-row').first();
	await expect(row.getByRole('button', { name: /^There/ })).toBeVisible();
	await expect(row.getByRole('button', { name: /^Absent/ })).toBeVisible();
	await expect(page.getByRole('button', { name: /^Coming/ })).toHaveCount(0);
});

test('"Mark as checked" makes each "no answer" absent and "Undo" brings them back', async ({
	page
}) => {
	const errors = watchConsole(page);
	await page.goto(gamePage);
	await expect(page.getByTestId('attendance-row').first()).toBeVisible();
	const players = await page.getByTestId('attendance-row').count();
	expect(players).toBeGreaterThan(0);
	await expect(page.getByTestId('attendance-count')).toContainText('no answer');

	await page.getByRole('button', { name: 'Mark as checked' }).click();
	const checked = page.getByTestId('mark-checked');
	await expect(checked).toHaveAttribute('data-done', 'true');
	await expect(checked).toContainText('Checked');
	await expect(page.getByTestId('settled')).toHaveText(
		/^\d+ players? with no answer (is|are) absent\.$/
	);
	await expect(page.getByTestId('attendance-row')).toHaveCount(0);
	await expect(page.getByTestId('closed-count')).toHaveText(/absent/);
	await expect(page.getByTestId('closed-count')).not.toHaveText(/no answer/);
	await expect(page.getByRole('button', { name: 'Mark as checked' })).toHaveCount(0);

	// The list opens behind a tap, with every player there or absent.
	await page.getByTestId('show-list').click();
	await expect(page.getByTestId('attendance-group').getByRole('heading')).not.toContainText([
		'No answer'
	]);

	await checked.getByRole('button', { name: 'Undo' }).click();
	await expect(page.getByTestId('mark-checked')).toHaveAttribute('data-done', 'false');
	await expect(page.getByRole('button', { name: 'Mark as checked' })).toBeVisible();
	// The list of a past game that is not checked is open again.
	await expect(page.getByTestId('attendance-count')).toContainText('no answer');
	expect(errors).toEqual([]);
});

test('a "Mark as checked" that fails shows "Not saved. Try again." and the stored state', async ({
	page
}) => {
	await page.goto(gamePage);
	await page.route('**/checked?**', (route) => route.abort());
	await page.getByRole('button', { name: 'Mark as checked' }).click();
	await expect(page.getByTestId('not-saved')).toHaveText('Not saved. Try again.');
	await expect(page.getByTestId('mark-checked')).toHaveAttribute('data-done', 'false');
	await expect(page.getByRole('button', { name: 'Mark as checked' })).toBeVisible();
});

test('a game without any answer warns before "Mark as checked"', async ({ page }) => {
	// The answer of the page as for a game where nobody answered.
	await page.route(`**/api/games/2026-2027/${AWAY_10_OCT}`, async (route) => {
		const answer = await (await route.fetch()).json();
		answer.attendance.count = { coming: 0, not_coming: 0, no_answer: 10 };
		answer.attendance.checked = false;
		await route.fulfill({ json: answer });
	});
	await page.goto(gamePage);
	await expect(page.getByTestId('no-answer-warning')).toHaveText(
		'Nobody has an answer yet: marking as checked makes all 10 players absent.'
	);
	await expect(page.getByRole('button', { name: 'Mark as checked' })).toBeVisible();
});

test('the row "Not checked" follows the taps in the list', async ({ page }) => {
	await page.goto(gamePage);
	const row = page.getByTestId('mark-checked');
	await expect(page.getByTestId('no-answer-warning')).toBeVisible();
	const player = page.getByTestId('attendance-row').first();
	await player.getByRole('button', { name: /^There/ }).click();
	await expect(row).toContainText('1 there');
	await expect(page.getByTestId('no-answer-warning')).toHaveCount(0);
	// A tap on the chosen state goes back to "no answer", and the warning comes back.
	await player.getByRole('button', { name: /^There/ }).click();
	await expect(row).not.toContainText('there');
	await expect(page.getByTestId('no-answer-warning')).toBeVisible();
});

test('an away game shows its drivers below "Not checked"', async ({ page }) => {
	const drivers = `/api/games/2026-2027/${AWAY_10_OCT}/drivers`;
	await page.request.put(drivers, { ...origin, data: { drivers: 1 } });
	await page.goto(gamePage);
	await expect(page.getByTestId('checked-drivers')).toHaveText(/^Drivers: /);
	await expectNoHorizontalScroll(page, 320);
	await expectAccessible(page);
});

test('a checked game shows "· Checked" on "Season"', async ({ page }) => {
	await page.request.put(checkedApi, origin);
	try {
		await page.goto('/season');
		await page.getByTestId('labels').getByRole('button', { name: 'Past', exact: true }).click();
		const row = page
			.getByTestId('part-past')
			.locator(`[data-testid="game"][data-id="${AWAY_10_OCT}"]`);
		await expect(row.getByTestId('game-checked')).toHaveText('· Checked');
	} finally {
		await page.request.delete(checkedApi, origin);
	}
	await page.reload();
	await page.getByTestId('labels').getByRole('button', { name: 'Past', exact: true }).click();
	await expect(
		page
			.getByTestId('part-past')
			.locator(`[data-testid="game"][data-id="${AWAY_10_OCT}"]`)
			.getByTestId('game-checked')
	).toHaveCount(0);
});

test('a checked game fits a phone and is accessible', async ({ page }) => {
	await page.request.put(checkedApi, origin);
	try {
		await page.setViewportSize({ width: 320, height: 700 });
		await page.goto(gamePage);
		await expect(page.getByTestId('mark-checked')).toHaveAttribute('data-done', 'true');
		await expectNoHorizontalScroll(page, 320);
		await expectAccessible(page);
	} finally {
		await page.request.delete(checkedApi, origin);
	}
});

test('"Attendance" counts a checked game and lists it as missed', async ({ page }) => {
	const errors = watchConsole(page);
	await page.request.put(checkedApi, origin);
	try {
		await page.setViewportSize({ width: 320, height: 700 });
		await page.goto('/attendance');
		await expect(page.getByTestId('games-from')).toHaveCount(0);
		// A player who had no answer: "Checked" made the player absent, without a reason.
		const row = page
			.getByTestId('player-totals')
			.filter({
				has: page.getByText(/^of 1 game: 0 there · 0 absent, valid · 1 absent, not valid$/)
			})
			.first();
		await row.getByRole('button').click();
		await expect(row.getByTestId('missed')).toContainText('Game Sat 10 Oct · without a reason');
		await expectNoHorizontalScroll(page, 320);
		await expectAccessible(page);
	} finally {
		await page.request.delete(checkedApi, origin);
	}
	await page.reload();
	await expect(page.getByTestId('totals-games')).toHaveCount(0);
	expect(errors).toEqual([]);
});

test('a player who missed two games on one date sees both', async ({ page }) => {
	const errors = watchConsole(page);
	// Two games of the team on one date: the answer is replaced.
	await page.route('**/api/attendance*', async (route) => {
		const overview = await (await route.fetch()).json();
		const game = { date: '2026-10-10', kind: 'game' };
		const row = {
			id: 1,
			first_name: 'Avery',
			last_name: null,
			there: 0,
			absent_valid: 0,
			absent_not_valid: 0,
			trainings: 0,
			games: { games: 2, there: 0, absent_valid: 1, absent_not_valid: 1 },
			missed: [
				{ ...game, reason: null, valid: false },
				{ ...game, reason: 'Ill', valid: true }
			]
		};
		await route.fulfill({ json: { ...overview, games: 2, team: [row], left: [] } });
	});
	await page.goto('/attendance');
	await page.getByTestId('player-totals').getByRole('button').click();
	await expect(page.getByTestId('missed').getByRole('listitem')).toHaveText([
		'Game Sat 10 Oct · without a reason',
		'Game Sat 10 Oct · Ill'
	]);
	expect(errors).toEqual([]);
});
