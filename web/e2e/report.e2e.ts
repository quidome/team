// The page "Report" (decision 0033). The server's clock is Monday 12 October 2026 (DEV_NOW). These
// tests run after all other tests of the logged-in server (one database): the report shows what
// they stored, so the tests compare the page with the answer of `/api/report`.
import { expect, test, type Page } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

test.describe.configure({ mode: 'serial' });

/** Writes through the API need the origin of the page. */
const origin = { headers: { Origin: 'http://localhost:4100' } };

type Share = {
	counted: number;
	there: number;
	absent_valid: number;
	absent_not_valid: number;
	percent: number;
	percent_valid: number;
	percent_not_valid: number;
} | null;
type Duty = {
	done: number;
	planned: number;
	done_by_other: number;
	done_for_other: number;
	not_done: number;
};
type Row = {
	player: number;
	name: string;
	trainings: Share;
	games: Share;
	duties: { drive: Duty; table: Duty; referee: Duty } | null;
};
type Report = {
	season: string;
	incomplete: {
		not_opened: number;
		first_not_opened: string | null;
		not_checked: number;
		first_not_checked: { season: string; id: number } | null;
	};
	team: Row[];
	left: Row[];
	copies: Record<'trainings' | 'games' | 'drive' | 'table' | 'referee', string | null>;
};

async function report(page: Page): Promise<Report> {
	return (await page.request.get('/api/report')).json();
}

/** The cells of a row as the page must show them. */
function cells(row: Row): string[] {
	const share = (value: Share) => (value ? `${value.percent}%` : '–');
	const duty = (key: 'drive' | 'table' | 'referee') =>
		row.duties ? `${row.duties[key].done} + ${row.duties[key].planned}` : '–';
	return [
		row.name,
		share(row.trainings),
		share(row.games),
		duty('drive'),
		duty('table'),
		duty('referee')
	];
}

test('the start page links to the report after "Duties"', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/');
	const links = page.getByTestId('team-links').getByRole('link');
	await expect(links).toHaveText([
		'Season',
		'Attendance',
		'Duties',
		'Report',
		'Team',
		'Players',
		'Settings'
	]);
	await links.filter({ hasText: 'Report' }).click();
	await expect(page.getByRole('heading', { level: 1 })).toHaveText('Report 2026-2027');
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL('/');
	expect(errors).toEqual([]);
});

test('the table shows the numbers of the report for each player', async ({ page }) => {
	const errors = watchConsole(page);
	const answer = await report(page);
	expect(answer.team.length, 'players in the team from the earlier tests').toBeGreaterThan(0);
	await page.goto('/report');
	const team = page.getByTestId('report-team').getByTestId('report-row');
	await expect(team).toHaveCount(answer.team.length);
	for (const [index, row] of answer.team.entries()) {
		await expect(team.nth(index).locator('th, td')).toHaveText(cells(row));
	}
	if (answer.left.length > 0) {
		await expect(page.getByRole('heading', { name: 'Left the team' })).toBeVisible();
		const left = page.getByTestId('report-left').getByTestId('report-row');
		for (const [index, row] of answer.left.entries()) {
			await expect(left.nth(index).locator('th, td')).toHaveText(cells(row));
		}
	} else {
		await expect(page.getByRole('heading', { name: 'Left the team' })).toHaveCount(0);
	}
	await expectAccessible(page);
	expect(errors).toEqual([]);
});

test('what does not count yet opens the oldest, and "Back" returns to the report', async ({
	page
}) => {
	const answer = await report(page);
	const { incomplete } = answer;
	await page.goto('/report');
	if (incomplete.not_opened === 0 && incomplete.not_checked === 0) {
		await expect(page.getByTestId('report-incomplete')).toHaveCount(0);
		return;
	}
	if (incomplete.not_opened > 0) {
		const link = page.getByTestId('report-not-opened');
		await expect(link).toContainText(`${incomplete.not_opened} training day`);
		await link.click();
		await expect(page).toHaveURL(
			new RegExp(`/training/${incomplete.first_not_opened}\\?from=report`)
		);
		await page.getByRole('link', { name: 'Back' }).click();
		await expect(page).toHaveURL('/report');
	}
	if (incomplete.not_checked > 0) {
		await page.getByTestId('report-not-checked').click();
		await expect(page).toHaveURL(
			new RegExp(
				`/game/${incomplete.first_not_checked?.season}/${incomplete.first_not_checked?.id}`
			)
		);
		await page.getByRole('link', { name: 'Back' }).click();
		await expect(page).toHaveURL('/report');
	}
});

test('the table fits on a phone with names of 10 characters', async ({ page }) => {
	const added = await page.request.post('/api/players', {
		...origin,
		data: {
			first_name: 'Wilhelmina',
			last_name: null,
			role: { role: 'plays_games', from: '2026-09-01' }
		}
	});
	expect(added.ok()).toBe(true);
	await page.goto('/report');
	await expect(page.getByTestId('report-team').getByText('Wilhelmina')).toBeVisible();
	await expectNoHorizontalScroll(page, 360);
});

/** The lines of the details of `row`, as the page must show them. */
function details(row: Row): string[] {
	const share = (value: NonNullable<Share>, one: string, more: string) =>
		`${value.counted} ${value.counted === 1 ? one : more}: ${value.there} there (${value.percent}%) · ` +
		`${value.absent_valid} absent, valid (${value.percent_valid}%) · ` +
		`${value.absent_not_valid} absent, not valid (${value.percent_not_valid}%)`;
	const lines = [row.name];
	if (row.trainings) lines.push(share(row.trainings, 'training', 'trainings'));
	if (row.games) lines.push(share(row.games, 'game', 'games'));
	for (const [key, name] of [
		['drive', 'Drive'],
		['table', 'Table'],
		['referee', 'Referee']
	] as const) {
		const total = row.duties?.[key];
		if (!total) continue;
		const corrections: [number, string][] = [
			[total.done_by_other, 'done by another player'],
			[total.done_for_other, 'done for another player'],
			[total.not_done, 'not done']
		];
		if (total.done + total.planned + corrections.reduce((sum, [n]) => sum + n, 0) === 0) continue;
		lines.push(`${name}: ${total.done} done · ${total.planned} planned`);
		for (const [n, text] of corrections) if (n > 0) lines.push(`${n} ${text}`);
	}
	if (lines.length === 1) lines.push('Nothing counts yet this season.');
	if (row.trainings || row.games) lines.push('Missed trainings and games');
	return lines;
}

test('a tap on a name opens the details, a second tap closes them', async ({ page }) => {
	const errors = watchConsole(page);
	const answer = await report(page);
	const [first, second] = answer.team;
	expect(second, 'two players in the team').toBeTruthy();
	await page.goto('/report');
	const team = page.getByTestId('report-team');
	const name = (row: Row) => team.getByRole('button', { name: row.name, exact: true });

	await name(first).click();
	await expect(name(first)).toHaveAttribute('aria-expanded', 'true');
	const shown = team.getByTestId('report-details');
	await expect(shown).toHaveCount(1);
	await expect(shown.locator('p')).toHaveText(details(first));

	// Two players open at the same time.
	await name(second).click();
	await expect(shown).toHaveCount(2);
	await expect(shown.nth(1).locator('p')).toHaveText(details(second));

	await name(first).click();
	await expect(name(first)).toHaveAttribute('aria-expanded', 'false');
	await expect(shown).toHaveCount(1);
	await expectAccessible(page);
	expect(errors).toEqual([]);
});

test('"Missed trainings and games" opens "Attendance" with the player open, "Back" returns', async ({
	page
}) => {
	const answer = await report(page);
	const row = answer.team.find((row) => row.trainings || row.games);
	expect(row, 'a player with trainings or games').toBeTruthy();
	await page.goto('/report');
	await page
		.getByTestId('report-team')
		.getByRole('button', { name: row!.name, exact: true })
		.click();
	await page.getByTestId('details-missed').click();
	await expect(page).toHaveURL(`/attendance?player=${row!.player}&from=report`);
	// Only that player's list is open.
	await expect(page.getByTestId('missed')).toHaveCount(1);
	await expect(page.locator('[aria-expanded="true"]')).toContainText(row!.name.split(' ')[0]);
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL('/report');
});

test('the details of a player who left open too', async ({ page }) => {
	// A player who played games in September only: under "Left the team".
	const added = await page.request.post('/api/players', {
		...origin,
		data: {
			first_name: 'Quirijn',
			last_name: null,
			role: { role: 'plays_games', from: '2026-09-01' }
		}
	});
	expect(added.ok(), await added.text()).toBe(true);
	const id = (await added.json()).player.id;
	const left = await page.request.post(`/api/players/${id}/team-leave`, {
		...origin,
		data: { from: '2026-10-01' }
	});
	expect(left.ok(), await left.text()).toBe(true);
	const answer = await report(page);
	const row = answer.left.find((row) => row.name === 'Quirijn');
	expect(row, 'Quirijn under "Left the team"').toBeTruthy();
	await page.goto('/report');
	const table = page.getByTestId('report-left');
	await table.getByRole('button', { name: row!.name, exact: true }).click();
	await expect(table.getByTestId('report-details').locator('p')).toHaveText(details(row!));
});

test('"Copy" puts the text of a column on the clipboard', async ({ page }) => {
	const errors = watchConsole(page);
	const answer = await report(page);
	await page.goto('/report');
	for (const [key, text] of Object.entries(answer.copies)) {
		const button = page.getByTestId(`copy-${key}`);
		if (text === null) {
			// A column without lines has no "Copy".
			await expect(button).toHaveCount(0);
			continue;
		}
		await button.click();
		await expect(page.getByTestId('copy-state')).toHaveText('Copied');
		expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(text);
	}
	// "Trainingen", from the team only, with today's date.
	expect(answer.copies.trainings).toMatch(
		/^Trainingen .*seizoen 2026-2027 \(tot en met 12 okt\)\n/
	);
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 360);
	expect(errors).toEqual([]);
});

test('when the browser refuses the clipboard, the text shows to select', async ({ page }) => {
	await page.addInitScript(() => {
		navigator.clipboard.writeText = () =>
			Promise.reject(new DOMException('Refused', 'NotAllowedError'));
	});
	const answer = await report(page);
	await page.goto('/report');
	await page.getByTestId('copy-trainings').click();
	await expect(page.getByTestId('copy-state').locator('p')).toHaveText(
		'Select the text and copy it.'
	);
	await expect(page.getByTestId('copy-text')).toHaveText(answer.copies.trainings!);
});

/** Changes the answer of `/api/report` for this page. */
async function reportAnswer(page: Page, change: (answer: Report) => Report) {
	await page.route('**/api/report', async (route) => {
		const response = await route.fetch();
		await route.fulfill({ response, json: change(await response.json()) });
	});
}

const wide: Duty = { done: 12, planned: 3, done_by_other: 0, done_for_other: 0, not_done: 0 };

test('every heading and cell fits its column on a phone', async ({ page }) => {
	await reportAnswer(page, (answer) => ({
		...answer,
		team: answer.team.map((row, index) =>
			index === 0
				? {
						...row,
						name: 'Wilhelmina',
						trainings: row.trainings ?? answer.team.find((r) => r.trainings)!.trainings,
						duties: { drive: wide, table: wide, referee: wide }
					}
				: row
		)
	}));
	await page.setViewportSize({ width: 360, height: 740 });
	await page.goto('/report');
	await expect(page.getByTestId('report-team')).toBeVisible();
	const overflowing = await page
		.getByTestId('report-team')
		.evaluate((table) =>
			[...table.querySelectorAll('thead th, tbody td, tbody th button')]
				.filter((cell) => cell.scrollWidth > cell.clientWidth)
				.map((cell) => cell.textContent?.trim())
		);
	expect(overflowing).toEqual([]);
	// The text keeps the same distance to the border on the left and on the right.
	const gaps = await page.getByTestId('report-team').evaluate((table) => {
		const card = table.parentElement!.getBoundingClientRect();
		const text = (cell: Element) => {
			const range = document.createRange();
			range.selectNodeContents(cell);
			return range.getBoundingClientRect();
		};
		const row = table.querySelector('tbody tr')!;
		const name = text(row.querySelector('th button')!);
		const last = text([...row.querySelectorAll('td')].at(-1)!);
		return { left: name.left - card.left, right: card.right - last.right };
	});
	expect(Math.abs(gaps.left - gaps.right), JSON.stringify(gaps)).toBeLessThanOrEqual(4);
	expect(gaps.right).toBeGreaterThanOrEqual(8);
	await expectNoHorizontalScroll(page, 360);
});

test('the answer to "Copy" shows above the table, close to the buttons', async ({ page }) => {
	await page.goto('/report');
	await page.getByTestId('copy-trainings').click();
	const state = await page.getByTestId('copy-state').boundingBox();
	const table = await page.getByTestId('report-team').boundingBox();
	expect(state!.y).toBeLessThan(table!.y);
});

test('"1 game not checked" opens the game, and "Back" returns to the report', async ({ page }) => {
	const season = await (await page.request.get('/api/season')).json();
	const past = season.past.find(
		(event: { game?: { reason: string } }) => event.game?.reason === 'team'
	);
	expect(past, 'a past game of the team').toBeTruthy();
	await reportAnswer(page, (answer) => ({
		...answer,
		incomplete: {
			...answer.incomplete,
			not_checked: 1,
			first_not_checked: { season: answer.season, id: past.game.id }
		}
	}));
	await page.goto('/report');
	const link = page.getByTestId('report-not-checked');
	await expect(link).toHaveText('1 game not checked');
	await link.click();
	await expect(page).toHaveURL(`/game/2026-2027/${past.game.id}?from=report`);
	await page.unroute('**/api/report');
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL('/report');
});

test('the details show the corrections after a game', async ({ page }) => {
	const correction: Duty = {
		done: 2,
		planned: 1,
		done_by_other: 1,
		done_for_other: 1,
		not_done: 1
	};
	const none: Duty = { done: 0, planned: 0, done_by_other: 0, done_for_other: 0, not_done: 0 };
	await reportAnswer(page, (answer) => ({
		...answer,
		team: answer.team.map((row, index) =>
			index === 0 ? { ...row, duties: { drive: none, table: correction, referee: none } } : row
		)
	}));
	await page.goto('/report');
	const first = page
		.getByTestId('report-team')
		.getByTestId('report-row')
		.first()
		.getByRole('button');
	await first.click();
	const duty = page.getByTestId('details-duty');
	await expect(duty).toHaveCount(1);
	await expect(duty.locator('p')).toHaveText([
		'Table: 2 done · 1 planned',
		'1 done by another player',
		'1 done for another player',
		'1 not done'
	]);
});

test('an empty season says so', async ({ page }) => {
	await reportAnswer(page, (answer) => ({
		...answer,
		incomplete: { not_opened: 0, first_not_opened: null, not_checked: 0, first_not_checked: null },
		team: [],
		left: []
	}));
	await page.goto('/report');
	await expect(page.getByTestId('report-empty')).toHaveText(
		'No trainings, games or duties yet this season.'
	);
	await expect(page.getByTestId('report-team')).toHaveCount(0);
});

test('a report that cannot be loaded says so', async ({ page }) => {
	await page.route('**/api/report', (route) => route.fulfill({ status: 500, body: 'error' }));
	await page.goto('/report');
	await expect(page.getByTestId('report-error')).toHaveText(
		'The report could not be loaded. Load the page again.'
	);
});
