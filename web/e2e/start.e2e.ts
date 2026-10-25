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

test('the start page shows the team, the open items, "Last", "Next" and the links', async ({
	page
}) => {
	const errors = watchConsole(page);
	await page.goto('/');
	await expect(page.getByTestId('chosen-team')).toHaveText('Blue Drakes M16 1');
	const parts = page.locator(
		'[data-testid="open-items"], [data-testid="last"], [data-testid="next"], [data-testid="team-links"]'
	);
	await expect(parts).toHaveCount(4);
	const order = await parts.evaluateAll((elements) =>
		elements.map((element) => element.getAttribute('data-testid'))
	);
	expect(order).toEqual(['open-items', 'last', 'next', 'team-links']);
	await expect(page.getByTestId('team-links').getByRole('link')).toHaveText([
		'Season',
		'Attendance',
		'Team',
		'Players',
		'Settings'
	]);
	// The old parts are gone.
	await expect(page.getByTestId('game-list-card')).toHaveCount(0);
	await expect(page.getByTestId('training-card')).toHaveCount(0);
	await expect(page.getByTestId('unseen-changes')).toHaveCount(0);
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([]);
});

test('without open items the start page says "Nothing open."', async ({ page }) => {
	await withOpenItems(page, []);
	await page.goto('/');
	await expect(page.getByTestId('nothing-open')).toHaveText('Nothing open.');
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

test('"Last" opens the last training, also after "Checked"', async ({ page }) => {
	await page.goto('/');
	const last = page.getByTestId('last');
	await expect(last.getByTestId('training-row')).toHaveCount(1);
	const date = await last.getByTestId('training-row').getAttribute('data-date');
	await last.getByTestId('training-row').getByRole('button').click();
	await expect(page).toHaveURL(new RegExp(`/training/${date}\\?from=start$`));
	if (await page.getByRole('button', { name: 'Checked', exact: true }).isVisible())
		await page.getByRole('button', { name: 'Checked', exact: true }).click();
	await expect(page.getByTestId('checked')).toHaveText('Checked');
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/$/);
	await expect(last.getByTestId('training-row')).toHaveAttribute('data-date', date ?? '');
	await expect(last.getByTestId('training-status')).toHaveText(/ · Checked$/);
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
