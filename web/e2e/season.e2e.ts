// The page "Season" (story A of start page and season). The server's clock is Monday 12 October
// 2026 (DEV_NOW), so the season is 2026-2027. These tests run after the training tests (one
// database, in this order): the games of the fake source and the Tuesday trainings exist.
import { expect, test } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

test.describe.configure({ mode: 'serial' });

function label(page: import('@playwright/test').Page, name: string) {
	return page.getByTestId('labels').getByRole('button', { name, exact: true });
}

test('without a team, "Season" shows the trainings and asks for the team', async ({ page }) => {
	// The choose tests end with a club saved without a team.
	await page.goto('/season');
	await expect(page.getByTestId('team-needed')).toHaveText('Choose the team to see its games.');
	await expect(page.getByTestId('part-coming').getByTestId('training-row').first()).toBeVisible();
	await expect(page.getByTestId('game')).toHaveCount(0);

	// Choose Blue Drakes M16 1 again for the next tests; the choice reads the game list.
	await page.goto('/choose');
	await page.getByRole('button', { name: 'Other club' }).click();
	await page.getByLabel('Club name').fill('drakes');
	await page.getByRole('button', { name: 'Search' }).click();
	await page.getByTestId('club-result').filter({ hasText: 'Blue Drakes' }).click();
	await page.getByTestId('team-choice').filter({ hasText: 'M16 1' }).click();
	await expect(page.getByTestId('chosen-team')).toHaveText('Blue Drakes M16 1');
	await expect(page.getByTestId('next')).toBeVisible();
});

test('"Season" is first in the links and opens with "Coming" on', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/');
	const links = page.getByTestId('team-links').getByRole('link');
	await expect(links).toHaveText(['Season', 'Attendance', 'Duties', 'Team', 'Players', 'Settings']);
	await links.first().click();

	await expect(page.getByRole('heading', { name: 'Season 2026-2027' })).toBeVisible();
	await expect(label(page, 'Coming')).toHaveAttribute('aria-pressed', 'true');
	for (const name of ['Past', 'Games of the team', 'Possible duty games', 'Trainings'])
		await expect(label(page, name)).toHaveAttribute('aria-pressed', 'false');
	await expect(page.getByTestId('part-coming')).toBeVisible();
	await expect(page.getByTestId('part-past')).toHaveCount(0);

	const coming = page.getByTestId('part-coming');
	await expect(coming.getByTestId('week').first()).toHaveText('Week 42 · 12 Oct – 18 Oct');
	await expect(coming.getByTestId('day').first()).toHaveText('Tue 13 Oct');
	await expect(coming.getByTestId('game').first()).toBeVisible();
	await expect(coming.getByTestId('training-row').first()).toBeVisible();
	expect(errors).toEqual([]);
});

test('labels of different groups narrow, labels of one group add', async ({ page }) => {
	await page.goto('/season');
	const coming = page.getByTestId('part-coming');

	await label(page, 'Possible duty games').click();
	await expect(label(page, 'Possible duty games')).toHaveAttribute('aria-pressed', 'true');
	await expect(coming.getByTestId('training-row')).toHaveCount(0);
	await expect(coming.getByTestId('game').first()).toBeVisible();
	await expect(coming.locator('[data-testid="game"][data-of-team="true"]')).toHaveCount(0);

	await label(page, 'Possible duty games').click();
	await label(page, 'Games of the team').click();
	await label(page, 'Trainings').click();
	await expect(coming.getByTestId('training-row').first()).toBeVisible();
	await expect(coming.locator('[data-testid="game"][data-of-team="true"]').first()).toBeVisible();
	await expect(coming.locator('[data-testid="game"][data-of-team="false"]')).toHaveCount(0);
});

test('"Past" with "Coming" shows two parts, past newest first', async ({ page }) => {
	await page.goto('/season');
	await label(page, 'Past').click();
	await expect(page.getByTestId('part-coming')).toBeVisible();
	const past = page.getByTestId('part-past');
	await expect(past).toBeVisible();
	const days = await past.getByTestId('day').allTextContents();
	expect(days.length).toBeGreaterThan(1);
	// The first past date is the newest: Sat 10 Oct is before Mon 12 Oct, Sat 3 Oct before that.
	expect(days[0]).toBe('Sat 10 Oct');

	// Only "Past": the part "Coming" goes.
	await label(page, 'Coming').click();
	await expect(page.getByTestId('part-coming')).toHaveCount(0);
	await expect(past).toBeVisible();
});

test('a combination with no events says so', async ({ page }) => {
	// The fake source has past possible duty games; this answer has none.
	await page.route('**/api/season', async (route) => {
		const answer = await (await route.fetch()).json();
		answer.past = answer.past.filter(
			(event: { game?: { reason: string } }) => event.game?.reason !== 'duty'
		);
		await route.fulfill({ json: answer });
	});
	await page.goto('/season');
	await label(page, 'Possible duty games').click();
	await label(page, 'Coming').click();
	await label(page, 'Past').click();
	await expect(page.getByTestId('part-past').getByTestId('part-empty')).toHaveText(
		'No past possible duty games.'
	);
});

test('the labels are not kept between visits', async ({ page }) => {
	await page.goto('/season');
	await label(page, 'Trainings').click();
	await label(page, 'Past').click();
	await page.getByRole('link', { name: 'Back' }).click();
	await page.getByTestId('team-links').getByRole('link', { name: 'Season' }).click();
	await expect(label(page, 'Coming')).toHaveAttribute('aria-pressed', 'true');
	await expect(label(page, 'Trainings')).toHaveAttribute('aria-pressed', 'false');
	await expect(label(page, 'Past')).toHaveAttribute('aria-pressed', 'false');
});

test('a tap on a training opens it', async ({ page }) => {
	await page.goto('/season');
	const row = page.getByTestId('part-coming').getByTestId('training-row').first();
	const date = await row.getAttribute('data-date');
	await row.getByRole('button').click();
	await expect(page).toHaveURL(new RegExp(`/training/${date}$`));
});

test('"Add training" on "Season"', async ({ page }) => {
	await page.goto('/season');
	await page.getByRole('button', { name: 'Add training' }).click();
	const form = page.getByTestId('add-training');
	await expect(form.getByLabel('Date')).toHaveAttribute('min', '2026-07-01');
	await expect(form.getByLabel('Date')).toHaveAttribute('max', '2027-06-30');
	await form.getByLabel('Date').fill('2026-11-28');
	await form.getByLabel('Start').fill('10:00');
	await form.getByLabel('End').fill('11:30');
	await form.getByRole('button', { name: 'Save' }).click();
	await expect(page).toHaveURL(/\/training\/2026-11-28$/);
	await expect(page.getByTestId('training-heading')).toHaveText('Sat 28 Nov · 10:00–11:30');

	// "Back" returns to "Season", where the training shows as a row.
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/season\?season=2026-2027$/);
	await expect(page.locator('[data-testid="training-row"][data-date="2026-11-28"]')).toContainText(
		'extra'
	);
});

test('"Cancel" on a training day not opened, and "Show cancelled"', async ({ page }) => {
	await page.goto('/season');
	const row = page.locator('[data-testid="training-row"][data-date="2026-12-01"]');
	await row.getByRole('button', { name: 'Cancel training of Tue 1 Dec' }).click();
	await expect(row.getByTestId('training-status')).toHaveText('Cancelled');

	await page.getByRole('switch', { name: 'Show cancelled' }).uncheck();
	await expect(row).toHaveCount(0);
	// The setting stays on this phone.
	await page.reload();
	await expect(page.getByRole('switch', { name: 'Show cancelled' })).not.toBeChecked();
	await expect(row).toHaveCount(0);
	await page.getByRole('switch', { name: 'Show cancelled' }).check();
	await expect(row.getByTestId('training-status')).toHaveText('Cancelled');
});

test('the "Game list" card reads, and the new games show at once', async ({ page }) => {
	// After the tap, "Season" answers with a renamed home team, so only a new load shows it.
	let read = false;
	await page.route('**/api/readings', async (route) => {
		read = true;
		await route.continue();
	});
	await page.route('**/api/season*', async (route) => {
		const answer = await (await route.fetch()).json();
		const first = answer.coming.find((event: { game?: unknown }) => event.game);
		if (read && first) first.game.home = 'Read again';
		await route.fulfill({ json: answer });
	});
	await page.goto('/season');
	const card = page.getByTestId('game-list-card');
	await expect(card).toBeVisible();
	await expect(page.getByText('Read again')).toHaveCount(0);
	await card.getByRole('button', { name: 'Read the game list' }).click();
	await expect(card.getByTestId('last-read')).toBeVisible();
	await expect(page.getByTestId('part-coming').getByText('Read again')).toBeVisible();
	await expect(page.getByTestId('season-load-error')).toHaveCount(0);
	await page.unrouteAll({ behavior: 'ignoreErrors' });

	// An earlier season has no card and no "Add training".
	await page.goto('/season?season=2025-2026');
	await expect(page.getByTestId('game-list-card')).toHaveCount(0);
	await expect(page.getByRole('button', { name: 'Add training' })).toHaveCount(0);
});

test('"Back" returns to the page the training was opened from', async ({ page }) => {
	// From the line of training days not opened on "Attendance".
	await page.goto('/attendance');
	await page.getByTestId('not-opened').getByRole('link').click();
	await expect(page).toHaveURL(/\/training\/\d{4}-\d{2}-\d{2}\?from=attendance$/);
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/attendance\?season=2026-2027$/);

	// "Change" to another date keeps where the training was opened from.
	await page.goto('/training/2026-11-28?from=start');
	await page.getByRole('button', { name: 'Change' }).click();
	const change = page.getByTestId('change-training');
	await change.getByLabel('Date').fill('2026-11-29');
	await change.getByRole('button', { name: 'Save' }).click();
	await expect(page).toHaveURL(/\/training\/2026-11-29\?from=start$/);
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/$/);
});

test('a return from a training keeps the labels', async ({ page }) => {
	await page.goto('/season');
	const label = (name: string) =>
		page.getByTestId('labels').getByRole('button', { name, exact: true });
	await label('Trainings').click();
	await label('Past').click();
	const row = page.getByTestId('part-past').getByTestId('training-row').first();
	await row.getByRole('button').first().click();
	await expect(page).toHaveURL(/\/training\//);

	// With "Back".
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/season\?season=2026-2027$/);
	await expect(label('Trainings')).toHaveAttribute('aria-pressed', 'true');
	await expect(label('Past')).toHaveAttribute('aria-pressed', 'true');

	// With the browser's back.
	await page
		.getByTestId('part-past')
		.getByTestId('training-row')
		.first()
		.getByRole('button')
		.first()
		.click();
	await expect(page).toHaveURL(/\/training\//);
	await page.goBack();
	await expect(label('Trainings')).toHaveAttribute('aria-pressed', 'true');
	await expect(label('Past')).toHaveAttribute('aria-pressed', 'true');

	// A new visit from the start page opens with "Coming" again.
	await page.goto('/');
	await page.getByTestId('team-links').getByRole('link', { name: 'Season' }).click();
	await expect(label('Coming')).toHaveAttribute('aria-pressed', 'true');
	await expect(label('Trainings')).toHaveAttribute('aria-pressed', 'false');
});

test('"Delete training" goes to "Season"', async ({ page }) => {
	await page.goto('/training/2026-11-29');
	page.once('dialog', (dialog) => void dialog.accept());
	await page.getByRole('button', { name: 'Delete training' }).click();
	await expect(page).toHaveURL(/\/season\?season=2026-2027$/);
	await expect(page.locator('[data-testid="training-row"][data-date="2026-11-29"]')).toHaveCount(0);
});

test('another season is in the address and kept after a reload', async ({ page }) => {
	await page.goto('/season?season=2025-2026');
	await expect(page.getByRole('heading', { name: 'Season 2025-2026' })).toBeVisible();
	await page.reload();
	await expect(page.getByRole('heading', { name: 'Season 2025-2026' })).toBeVisible();
	await expect(page.getByTestId('season-empty')).toHaveText(
		'No games or trainings in 2025-2026 yet.'
	);
	await expect(page.getByTestId('no-trainings')).toHaveText(
		'No training days or trainings this season. Add a weekly training in Settings.'
	);
	await page.getByTestId('season-choice').selectOption('2026-2027');
	await expect(page).toHaveURL(/\/season\?season=2026-2027$/);
	await expect(page.getByRole('heading', { name: 'Season 2026-2027' })).toBeVisible();
});

test('a season that cannot be read', async ({ page }) => {
	await page.goto('/season?season=abc');
	await expect(page.getByTestId('invalid-season')).toHaveText('This season cannot be read.');
	await page.getByRole('link', { name: 'Current season' }).click();
	await expect(page.getByRole('heading', { name: 'Season 2026-2027' })).toBeVisible();
});

test('the page fits a small phone and is accessible', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/season');
	await label(page, 'Past').click();
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([]);
});
