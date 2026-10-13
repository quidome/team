// The list of changes (story 4). The real list depends on readings that the fake source cannot
// give (its answer never changes), so these tests replace the answers of GET /api/changes and
// POST /api/changes/seen; the server tests cover the real flow from a reading to a stored change.
import { expect, test, type Page } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

type Game = {
	date: string;
	start_time: string | null;
	home: string;
	away: string;
	hall: string;
	town: string;
};

function game(home: string, away: string, start_time: string | null = '16:00'): Game {
	return { date: '2026-10-31', start_time, home, away, hall: 'Bulwijk', town: 'Woerden' };
}

const movedTeamGame = {
	id: 1,
	version: 2,
	kind: 'moved',
	differences: ['start_time'],
	reason: 'team',
	before: game('Blue Drakes M16 1', 'Archipel M16 1', '16:00'),
	after: game('Blue Drakes M16 1', 'Archipel M16 1', '17:30')
};

const knownDutyGame = {
	id: 2,
	version: 1,
	kind: 'start_time_known',
	differences: ['start_time_known'],
	reason: 'duty',
	before: game('Blue Drakes X12 1', 'De Dunckers X12 1', null),
	after: game('Blue Drakes X12 1', 'De Dunckers X12 1', '11:30')
};

function newGame(id: number, reason: 'team' | 'duty') {
	const home = reason === 'team' ? 'Blue Drakes M16 1' : `Blue Drakes X14 ${id}`;
	return {
		id,
		version: 1,
		kind: 'new',
		differences: [],
		reason,
		before: null,
		after: game(home, `Opponent ${id}`)
	};
}

/** Serves `changes` as the list; "seen" removes what it gets. Gives the bodies of "seen". */
async function serve(page: Page, changes: object[]): Promise<object[]> {
	let list = [...changes] as { id: number }[];
	const seen: object[] = [];
	await page.route('**/api/changes', (route) => route.fulfill({ json: { changes: list } }));
	await page.route('**/api/changes/seen', async (route) => {
		const body = route.request().postDataJSON() as { changes: { id: number }[] };
		seen.push(body);
		const ids = body.changes.map((change) => change.id);
		list = list.filter((change) => !ids.includes(change.id));
		await route.fulfill({ json: { open: list.length } });
	});
	return seen;
}

test('a moved team game shows the team look and its old and new start time', async ({ page }) => {
	const errors = watchConsole(page);
	const seen = await serve(page, [movedTeamGame, knownDutyGame]);
	await page.goto('/changes');

	const changes = page.getByTestId('change');
	await expect(changes).toHaveCount(2);
	const moved = changes.first();
	await expect(moved).toHaveAttribute('data-of-team', 'true');
	await expect(moved.getByText('Team', { exact: true })).toBeVisible();
	await expect(moved.getByTestId('change-kind')).toHaveText('moved');
	await expect(moved.getByTestId('change-detail')).toHaveText(
		'Sat 31 Oct 16:00 → Sat 31 Oct 17:30'
	);
	await expect(changes.nth(1).getByTestId('change-kind')).toHaveText('start time known');
	await expect(changes.nth(1).getByTestId('change-detail')).toHaveText('Sat 31 Oct 11:30');

	await moved.getByRole('button', { name: /^Seen/ }).click();
	await expect(changes).toHaveCount(1);
	expect(seen).toEqual([{ changes: [{ id: 1, version: 2 }] }]);
	await expectAccessible(page);
	expect(errors).toEqual([]);
});

test('the filter shows only the changes of possible duty games', async ({ page }) => {
	const seen = await serve(page, [movedTeamGame, knownDutyGame]);
	await page.goto('/changes');
	await page.getByRole('radio', { name: 'Possible duty games' }).click();
	await expect(page.getByTestId('change')).toHaveCount(1);
	await expect(page.locator('[data-testid="change"][data-of-team="true"]')).toHaveCount(0);

	await page.getByRole('button', { name: 'Mark this one as seen' }).click();
	expect(seen).toEqual([{ changes: [{ id: 2, version: 1 }] }]);
	await expect(page.getByTestId('no-changes')).toHaveText(
		'No changes of possible duty games not seen.'
	);
	await page.getByRole('radio', { name: 'All' }).click();
	await expect(page.getByTestId('change')).toHaveCount(1);
});

test('5 new possible duty games show as one line that opens', async ({ page }) => {
	await serve(
		page,
		[1, 2, 3, 4, 5].map((id) => newGame(id, 'duty'))
	);
	await page.goto('/changes');
	await expect(page.getByTestId('change')).toHaveCount(0);
	const summary = page.getByTestId('new-games-summary');
	await expect(summary).toContainText('5 new games');
	await summary.click();
	await expect(page.getByTestId('new-games').getByTestId('change')).toHaveCount(5);
});

test('4 new possible duty games and 5 new games of the team show one by one', async ({ page }) => {
	await serve(page, [
		...[1, 2, 3, 4].map((id) => newGame(id, 'duty')),
		...[11, 12, 13, 14, 15].map((id) => newGame(id, 'team'))
	]);
	await page.goto('/changes');
	await expect(page.getByTestId('new-games-summary')).toHaveCount(0);
	await expect(page.getByTestId('change')).toHaveCount(9);
	await expect(page.locator('[data-testid="change"][data-of-team="true"]')).toHaveCount(5);
});

test('mark all as seen sends every change, also those in a closed summary line', async ({
	page
}) => {
	const seen = await serve(page, [
		movedTeamGame,
		...[11, 12, 13, 14, 15].map((id) => newGame(id, 'duty'))
	]);
	await page.goto('/changes');
	await expect(page.getByTestId('new-games')).toHaveCount(0);
	await page.getByRole('button', { name: 'Mark all as seen' }).click();
	expect(seen).toEqual([
		{
			changes: [1, 11, 12, 13, 14, 15].map((id) => ({ id, version: id === 1 ? 2 : 1 }))
		}
	]);
	await expect(page.getByTestId('no-changes')).toHaveText('No changes not seen.');
});

test('the page fits a small phone, and screenshots for the look', async ({ page }) => {
	const errors = watchConsole(page);
	await serve(page, [
		movedTeamGame,
		knownDutyGame,
		{
			id: 3,
			version: 1,
			kind: 'gone',
			differences: [],
			reason: 'duty',
			before: game('Blue Drakes M18 1', 'Hoops M18 1', '14:00'),
			after: null
		},
		{ ...newGame(4, 'duty'), kind: 'into_schedule' }
	]);
	await page.goto('/changes');
	await expect(page.getByTestId('change')).toHaveCount(4);
	await expect(page.getByTestId('change-kind').nth(2)).toHaveText('no longer in the game list');
	await expect(page.getByTestId('change-kind').nth(3)).toHaveText(
		'now a home game of another club team'
	);
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	await page.setViewportSize({ width: 390, height: 844 });
	for (const colorScheme of ['light', 'dark'] as const) {
		await page.emulateMedia({ colorScheme });
		await page.reload();
		await expect(page.getByTestId('change')).toHaveCount(4);
		await page.screenshot({ path: `test-results/look/changes-${colorScheme}.png`, fullPage: true });
	}
	expect(errors).toEqual([]);
});

test('with the filter on the team, the button marks only the changes of the team', async ({
	page
}) => {
	const seen = await serve(page, [movedTeamGame, knownDutyGame, newGame(11, 'team')]);
	await page.goto('/changes');
	await page.getByRole('radio', { name: 'Games of the team' }).click();
	await expect(page.getByTestId('change')).toHaveCount(2);
	await page.getByRole('button', { name: 'Mark these 2 as seen' }).click();
	expect(seen).toEqual([
		{
			changes: [
				{ id: 1, version: 2 },
				{ id: 11, version: 1 }
			]
		}
	]);
	await expect(page.getByTestId('no-changes')).toHaveText('No changes of the team not seen.');
});
