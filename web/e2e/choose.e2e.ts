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

test('without a team "Duties" still offers the overviews, and they say so', async ({ page }) => {
	await page.goto('/duties');
	await page.getByTestId('overviews').getByRole('link', { name: 'Driving overview' }).click();
	await expect(page.getByTestId('no-message')).toHaveText('No overview: no team chosen.');
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

	// Choosing the team starts the first reading. The start page: only the home games of the next
	// 4 weeks are open (schedule not entered yet; a first reading records no changes, and there are
	// no trainings), and in "Next" the first game of the team, because the two weeks from Mon 12 Oct
	// have none.
	await expect(page.getByTestId('chosen-team')).toHaveText('Blue Drakes M16 1');
	await expect(page.getByRole('link', { name: 'Change' })).toBeVisible();
	await expect(page.getByTestId('open-item')).toHaveText([
		/^\d+ home games: schedule not entered$/
	]);
	const next = page.getByTestId('next');
	await expect(next.getByTestId('day')).toHaveText(['Sat 31 Oct']);
	await expect(next.locator('[data-testid="game"][data-of-team="true"]')).toHaveCount(1);
	await expectAccessible(page);

	// The "Game list" card is on "Season".
	await page.goto('/season');
	await expect(page.getByTestId('last-read')).toHaveText(
		/^Last read: \w{3} \d{1,2} \w+ \d{4} \d{2}:\d{2}$/
	);
	expect(errors).toEqual([]);
});

test('reading the game list again has no changes, and the games stay', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/season');
	const games = page.getByTestId('part-coming').getByTestId('game');
	await expect(page.getByTestId('last-read')).toBeVisible();
	await expect(page.getByTestId('reading-result')).toHaveCount(0);
	await expect(games).toHaveCount(23);
	await page.getByRole('button', { name: 'Read the game list' }).click();
	await expect(page.getByTestId('reading-result')).toHaveText('No changes');
	await expect(games).toHaveCount(23);
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
	await page.goto('/season');
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

test('the open item of the game list goes to its card on "Season"', async ({ page }) => {
	// The server reads well each time, so its answer is replaced.
	await page.route('**/api/start', async (route) => {
		const start = await (await route.fetch()).json();
		start.open_items = [{ kind: 'out_of_date', since: '2026-10-09' }];
		await route.fulfill({ json: start });
	});
	await page.goto('/');
	const item = page.getByTestId('open-item');
	await expect(item).toHaveText('The games may be out of date since Fri 9 Oct.');
	await item.click();
	await expect(page).toHaveURL(/\/season#game-list$/);
	await expect(page.getByTestId('game-list-card')).toBeInViewport();
});

// The server's clock is 12 October 2026 (DEV_NOW): the games of M16 1 are 23 coming games (5 of
// the team) and 14 past games, on "Season" (no trainings yet in these tests).
test.describe('the games on "Season"', () => {
	const label = (page: import('@playwright/test').Page, name: string) =>
		page.getByTestId('labels').getByRole('button', { name, exact: true });
	const coming = (page: import('@playwright/test').Page) =>
		page.getByTestId('part-coming').getByTestId('game');

	test.beforeEach(async ({ page }) => {
		await page.goto('/season');
	});

	test('shows the coming games grouped by date, with the team marked', async ({ page }) => {
		const days = page.getByTestId('part-coming').getByTestId('day');
		await expect(days).toHaveCount(12);
		await expect(days.first()).toHaveText('Sat 31 Oct');
		await expect(coming(page)).toHaveCount(23);
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

	test('the labels show games of the team or possible duty games', async ({ page }) => {
		await label(page, 'Games of the team').click();
		await expect(coming(page)).toHaveCount(5);
		await expect(page.locator('[data-testid="game"][data-of-team="false"]')).toHaveCount(0);

		await label(page, 'Games of the team').click();
		await label(page, 'Possible duty games').click();
		await expect(coming(page)).toHaveCount(18);
		await expect(page.locator('[data-testid="game"][data-of-team="true"]')).toHaveCount(0);

		// Both labels of one group add up.
		await label(page, 'Games of the team').click();
		await expect(coming(page)).toHaveCount(23);
	});

	test('past games show with "Past"', async ({ page }) => {
		await expect(page.getByTestId('part-past')).toHaveCount(0);
		await label(page, 'Past').click();
		await expect(page.getByTestId('part-past').getByTestId('game')).toHaveCount(14);
		await expect(page.getByTestId('game')).toHaveCount(37);
		await label(page, 'Past').click();
		await expect(page.getByTestId('game')).toHaveCount(23);
	});

	test('labels that leave no games say so', async ({ page }) => {
		// As from mid-December, when the team has no coming games until the second half appears.
		await page.route('**/api/season*', async (route) => {
			const response = await route.fetch();
			const body = await response.json();
			body.coming = body.coming.filter(
				(event: { game?: { reason: string } }) => event.game?.reason === 'duty'
			);
			await route.fulfill({ response, json: body });
		});
		await page.goto('/season');
		await expect(coming(page)).toHaveCount(18);
		await label(page, 'Games of the team').click();
		await expect(coming(page)).toHaveCount(0);
		await expect(page.getByTestId('part-coming').getByTestId('part-empty')).toHaveText(
			'No coming games of the team.'
		);
		await label(page, 'Past').click();
		await expect(page.getByTestId('part-past').getByTestId('game')).toHaveCount(5);
	});

	test('games that cannot be loaded after a reading do not look like a failed reading', async ({
		page
	}) => {
		// Only the load after the tap fails: the page has loaded its games first.
		await expect(coming(page)).toHaveCount(23);
		await page.route('**/api/season*', (route) => route.fulfill({ status: 500, json: {} }));
		await page.getByRole('button', { name: 'Read the game list' }).click();
		await expect(page.getByTestId('reading-result')).toHaveText('No changes');
		await expect(page.getByTestId('season-load-error')).toBeVisible();
		await expect(page.getByText('Something went wrong')).toHaveCount(0);
		await expect(coming(page)).toHaveCount(23);
	});

	test('is accessible, has no console errors and fits a small phone', async ({ page }) => {
		const errors = watchConsole(page);
		await page.reload();
		await label(page, 'Past').click();
		await expect(page.getByTestId('game')).toHaveCount(37);
		await expectAccessible(page);
		await expectNoHorizontalScroll(page, 320);
		expect(errors).toEqual([]);
	});

	test('screenshots for the look', async ({ page }) => {
		for (const colorScheme of ['light', 'dark'] as const) {
			await page.emulateMedia({ colorScheme });
			await page.reload();
			await expect(coming(page)).toHaveCount(23);
			await page.screenshot({
				path: `test-results/look/season-${colorScheme}.png`,
				fullPage: true
			});
		}
	});
});

test.describe('the open items of changes', () => {
	test('without changes not seen, there is no open item for them', async ({ page }) => {
		await page.goto('/');
		await expect(page.locator('[data-kind="schedule_not_entered"]')).toBeVisible();
		await expect(page.locator('[data-kind="changes"]')).toHaveCount(0);
	});

	test('changes not seen are an open item that opens the list', async ({ page }) => {
		await page.route('**/api/start', async (route) => {
			const start = await (await route.fetch()).json();
			start.open_items = [{ kind: 'changes', count: 3 }];
			await route.fulfill({ json: start });
		});
		await page.goto('/');
		const item = page.locator('[data-kind="changes"]');
		await expect(item).toHaveText('3 changes not seen');
		await item.click();
		await expect(page).toHaveURL('/changes');
		await expect(page.getByRole('heading', { name: 'Changes' })).toBeVisible();
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
	// The games follow the new team: on "Season" its games are the team games now.
	await page.goto('/season');
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
