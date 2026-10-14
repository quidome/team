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
	await expect(page.getByTestId('game')).toHaveCount(0);
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

	// Choosing the team starts the first reading: the schedule shows without a tap.
	await expect(page.getByTestId('chosen-team')).toHaveText('Blue Drakes M16 1');
	await expect(page.getByRole('link', { name: 'Change' })).toBeVisible();
	await expect(page.getByTestId('game')).toHaveCount(23);
	await expect(page.getByTestId('day').first()).toHaveText('Sat 31 Oct');
	await expect(page.getByTestId('last-read')).toHaveText(
		/^Last read: \w{3} \d{1,2} \w+ \d{4} \d{2}:\d{2}$/
	);
	await expectAccessible(page);
	expect(errors).toEqual([]);
});

test('reading the game list again has no changes, and the schedule stays', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/');
	await expect(page.getByTestId('last-read')).toBeVisible();
	await expect(page.getByTestId('reading-result')).toHaveCount(0);
	await expect(page.getByTestId('game')).toHaveCount(23);
	await page.getByRole('button', { name: 'Read the game list' }).click();
	await expect(page.getByTestId('reading-result')).toHaveText('No changes');
	await expect(page.getByTestId('game')).toHaveCount(23);
	await expect(page.getByRole('button', { name: 'Read the game list' })).toBeEnabled();
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([]);
});

test('after 3 nights without a good reading the card says the games may be out of date', async ({
	page
}) => {
	// The fake source never fails, so the answer of the latest readings is replaced (story 5).
	const errors = watchConsole(page);
	await page.route('**/api/readings/latest', async (route) => {
		const latest = await (await route.fetch()).json();
		await route.fulfill({ json: { ...latest, out_of_date_since: '2026-10-09T01:00:05Z' } });
	});
	await page.goto('/');
	const warning = page.getByTestId('out-of-date');
	await expect(warning).toHaveText('The games may be out of date since Fri 9 October 2026 03:00.');
	await expect(warning).toHaveAttribute('role', 'alert');
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);

	await page.getByRole('button', { name: 'Read the game list' }).click();
	await expect(page.getByTestId('reading-result')).toHaveText('No changes');
	await expect(warning).toHaveCount(0);
	expect(errors).toEqual([]);
});

// The server's clock is 12 October 2026 (DEV_NOW): the schedule of M16 1 has 23 coming games
// (5 of the team) and 14 past games.
test.describe('the schedule', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/');
	});

	test('shows the coming games grouped by date, with the team marked', async ({ page }) => {
		const days = page.getByTestId('day');
		await expect(days).toHaveCount(12);
		await expect(days.first()).toHaveText('Sat 31 Oct');
		await expect(page.getByTestId('game')).toHaveCount(23);
		const teamGames = page.locator('[data-testid="game"][data-of-team="true"]');
		await expect(teamGames).toHaveCount(5);
		await expect(teamGames.getByText('Team', { exact: true })).toHaveCount(5);
	});

	test('a game without a known start time says so, last in its day', async ({ page }) => {
		const firstDay = page.getByRole('region', { name: 'Sat 31 Oct' }).getByTestId('game');
		await expect(firstDay).toHaveCount(4);
		await expect(firstDay.first()).toContainText('14:30');
		await expect(firstDay.last()).toContainText('time not known yet');
		await expect(firstDay.last()).toContainText('Blue Drakes X12 1');
	});

	test('the filter shows games of the team or possible duty games', async ({ page }) => {
		await page.getByRole('radio', { name: 'Games of the team' }).click();
		await expect(page.getByTestId('game')).toHaveCount(5);
		await expect(page.locator('[data-testid="game"][data-of-team="false"]')).toHaveCount(0);

		await page.getByRole('radio', { name: 'Possible duty games' }).click();
		await expect(page.getByTestId('game')).toHaveCount(18);
		await expect(page.locator('[data-testid="game"][data-of-team="true"]')).toHaveCount(0);

		await page.getByRole('radio', { name: 'All' }).click();
		await expect(page.getByTestId('game')).toHaveCount(23);
	});

	test('past games open on request', async ({ page }) => {
		await expect(page.getByTestId('past-games')).toHaveCount(0);
		await page.getByRole('button', { name: 'Show past games' }).click();
		await expect(page.getByTestId('past-games').getByTestId('game')).toHaveCount(14);
		await expect(page.getByTestId('game')).toHaveCount(37);
		await page.getByRole('button', { name: 'Hide past games' }).click();
		await expect(page.getByTestId('game')).toHaveCount(23);
	});

	test('a filter that leaves no games says so', async ({ page }) => {
		// As from mid-December, when the team has no coming games until the second half appears.
		await page.route('**/api/schedule', async (route) => {
			const response = await route.fetch();
			const body = await response.json();
			body.schedule.coming = body.schedule.coming.filter(
				(game: { reason: string }) => game.reason === 'duty'
			);
			await route.fulfill({ response, json: body });
		});
		await page.goto('/');
		await expect(page.getByTestId('game')).toHaveCount(18);
		await page.getByRole('radio', { name: 'Games of the team' }).click();
		await expect(page.getByTestId('game')).toHaveCount(0);
		await expect(page.getByTestId('no-coming-games')).toHaveText('No coming games of the team.');
		await page.getByRole('button', { name: 'Show past games' }).click();
		await expect(page.getByTestId('game')).toHaveCount(5);
	});

	test('a schedule that cannot be loaded after a reading does not look like a failed reading', async ({
		page
	}) => {
		// Only the reload after the tap fails: the page has loaded its schedule first.
		await expect(page.getByTestId('game')).toHaveCount(23);
		await page.route('**/api/schedule', (route) => route.fulfill({ status: 500, json: {} }));
		await page.getByRole('button', { name: 'Read the game list' }).click();
		await expect(page.getByTestId('reading-result')).toHaveText('No changes');
		await expect(page.getByTestId('schedule-error')).toBeVisible();
		await expect(page.getByText('Something went wrong')).toHaveCount(0);
		await expect(page.getByTestId('game')).toHaveCount(23);
	});

	test('without changes not seen, the start page has no line for them', async ({ page }) => {
		await expect(page.getByTestId('game')).toHaveCount(23);
		await expect(page.getByTestId('unseen-changes')).toHaveCount(0);
	});

	test('the line of changes not seen opens the list, and the next games stay in view', async ({
		page
	}) => {
		await page.route('**/api/schedule', async (route) => {
			const response = await route.fetch();
			const body = await response.json();
			body.schedule.unseen_changes = 3;
			await route.fulfill({ response, json: body });
		});
		await page.setViewportSize({ width: 390, height: 800 });
		await page.goto('/');
		const line = page.getByTestId('unseen-changes');
		await expect(line).toHaveText('3 changes not seen');
		const firstGame = await page.getByTestId('game').first().boundingBox();
		expect(firstGame && firstGame.y + firstGame.height).toBeLessThanOrEqual(800);
		await line.click();
		await expect(page).toHaveURL('/changes');
		await expect(page.getByRole('heading', { name: 'Changes' })).toBeVisible();
	});

	test('is accessible, has no console errors and fits a small phone', async ({ page }) => {
		const errors = watchConsole(page);
		await page.reload();
		await page.getByRole('button', { name: 'Show past games' }).click();
		await expect(page.getByTestId('game')).toHaveCount(37);
		await expectAccessible(page);
		await expectNoHorizontalScroll(page, 320);
		expect(errors).toEqual([]);
	});

	test('screenshots for the look', async ({ page }) => {
		for (const colorScheme of ['light', 'dark'] as const) {
			await page.emulateMedia({ colorScheme });
			await page.reload();
			await expect(page.getByTestId('game')).toHaveCount(23);
			await page.screenshot({
				path: `test-results/look/schedule-${colorScheme}.png`,
				fullPage: true
			});
		}
	});
});

test('the coordinator changes the team later', async ({ page }) => {
	await page.goto('/');
	await page.getByRole('link', { name: 'Change' }).click();
	await expect(page).toHaveURL('/choose');
	await expect(page.getByRole('heading', { name: 'Blue Drakes' })).toBeVisible();
	await page.getByTestId('team-choice').filter({ hasText: 'M16 2' }).click();

	await expect(page).toHaveURL('/');
	await expect(page.getByTestId('chosen-team')).toHaveText('Blue Drakes M16 2');
	// The schedule follows the new team: its games are the team games now.
	const teamGames = page.locator('[data-testid="game"][data-of-team="true"]');
	await expect(teamGames).toHaveCount(5);
	await expect(teamGames.filter({ hasText: 'Blue Drakes M16 2' })).toHaveCount(5);
	await expect(teamGames.filter({ hasText: 'Blue Drakes M16 1' })).toHaveCount(0);
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
	await expect(page.getByTestId('game')).toHaveCount(0);
});
