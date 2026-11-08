// "Checked" on the page of a game (story C of attendance at games). The server's clock is Monday
// 12 October 2026 (DEV_NOW), which is also the day of the release of "Checked" in the test
// database (`global-setup.ts`). These tests check a past game that other tests read, so they run
// last (project `checked`), and each test undoes its check.
import { expect, test } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

test.describe.configure({ mode: 'serial' });

/** The away game of the team on Saturday 10 October 2026 (past, before the release). */
const AWAY_10_OCT = 1073147;
const gamePage = `/game/2026-2027/${AWAY_10_OCT}`;
const checkedApi = `/api/games/2026-2027/${AWAY_10_OCT}/checked`;
const origin = { headers: { Origin: 'http://localhost:4100' } };

test('a game from before the release shows its list closed, with "There" and "Absent"', async ({
	page
}) => {
	await page.goto(gamePage);
	await expect(page.getByTestId('closed-list')).toBeVisible();
	await expect(page.getByTestId('attendance-row')).toHaveCount(0);
	await page.getByTestId('show-list').click();
	const row = page.getByTestId('attendance-row').first();
	await expect(row.getByRole('button', { name: /^There/ })).toBeVisible();
	await expect(row.getByRole('button', { name: /^Absent/ })).toBeVisible();
	await expect(page.getByRole('button', { name: /^Coming/ })).toHaveCount(0);
});

test('"Checked" makes each "no answer" absent and "Undo check" brings them back', async ({
	page
}) => {
	const errors = watchConsole(page);
	await page.goto(gamePage);
	await page.getByTestId('show-list').click();
	const players = await page.getByTestId('attendance-row').count();
	expect(players).toBeGreaterThan(0);
	await expect(page.getByTestId('attendance-count')).toContainText('no answer');

	await page.getByRole('button', { name: 'Checked', exact: true }).click();
	const checked = page.getByTestId('checked');
	await expect(checked).toContainText('Checked');
	await expect(page.getByTestId('settled')).toHaveText(
		/^\d+ players? with no answer (is|are) absent\.$/
	);
	await expect(page.getByTestId('attendance-row')).toHaveCount(0);
	await expect(page.getByTestId('closed-count')).toHaveText(/absent/);
	await expect(page.getByTestId('closed-count')).not.toHaveText(/no answer/);
	await expect(page.getByRole('button', { name: 'Checked', exact: true })).toHaveCount(0);

	// The list opens behind a tap, with every player there or absent.
	await page.getByTestId('show-list').click();
	await expect(page.getByTestId('attendance-group').getByRole('heading')).not.toContainText([
		'No answer'
	]);

	await checked.getByRole('button', { name: 'Undo check' }).click();
	await expect(page.getByTestId('checked')).toHaveCount(0);
	await expect(page.getByRole('button', { name: 'Checked', exact: true })).toBeVisible();
	await page.getByTestId('show-list').click();
	await expect(page.getByTestId('attendance-count')).toContainText('no answer');
	expect(errors).toEqual([]);
});

test('a "Checked" that fails shows "Not saved. Try again." and the stored state', async ({
	page
}) => {
	await page.goto(gamePage);
	await page.route('**/checked', (route) => route.abort());
	await page.getByRole('button', { name: 'Checked', exact: true }).click();
	await expect(page.getByTestId('not-saved')).toHaveText('Not saved. Try again.');
	await expect(page.getByTestId('checked')).toHaveCount(0);
	await expect(page.getByRole('button', { name: 'Checked', exact: true })).toBeVisible();
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
		await expect(page.getByTestId('checked')).toBeVisible();
		await expectNoHorizontalScroll(page, 320);
		await expectAccessible(page);
	} finally {
		await page.request.delete(checkedApi, origin);
	}
});
