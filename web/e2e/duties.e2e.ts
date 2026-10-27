// The page of a game (story A of duties). The server's clock is Monday 12 October 2026 (DEV_NOW).
// These tests run after the tests of the start page (one database, in this order): Blue Drakes
// M16 1 is chosen, with players and trainings from the earlier tests.
import { expect, test, type Page } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

test.describe.configure({ mode: 'serial' });

type Places = {
	schedule?: { required: boolean } | null;
	drivers?: { required: boolean; proposal: number } | null;
};
type Game = { id: number; home: string; away: string; reason: string; places?: Places | null };

/** The coming games of the current season, from the answer of "Season". */
async function comingGames(page: Page): Promise<Game[]> {
	const answer = await (await page.request.get('/api/season')).json();
	return answer.coming
		.filter((event: { game?: Game }) => event.game)
		.map((event: { game: Game }) => event.game);
}

async function findGame(page: Page, pick: (game: Game) => boolean): Promise<Game> {
	const game = (await comingGames(page)).find(pick);
	expect(game, 'a game for this test').toBeTruthy();
	return game!;
}

/** A home game of another club team: 4 places to choose. */
const otherHomeGame = (game: Game) => game.reason === 'duty' && !!game.places?.schedule;
/** An away game of the team at another club: drivers. */
const awayGame = (game: Game) => game.reason === 'team' && !!game.places?.drivers;

const place = (page: Page, name: string) =>
	page.getByTestId('schedule').getByRole('combobox', { name, exact: true });

test('a home game of the club has 4 places and "Schedule entered"', async ({ page }) => {
	const errors = watchConsole(page);
	const game = await findGame(page, otherHomeGame);
	await page.goto(`/game/2026-2027/${game.id}`);
	for (const name of ['Referee 1', 'Referee 2', 'Table 1', 'Table 2'])
		await expect(place(page, name)).toHaveValue('');
	await expect(page.getByTestId('team-places')).toHaveText('No places for the team.');
	await expect(page.getByTestId('schedule-state')).toHaveText('Schedule not entered');

	// Each choice saves at once.
	await place(page, 'Referee 1').selectOption({ label: 'M16 1' });
	await expect(page.getByTestId('team-places')).toHaveText('M16 1: Referee 1');
	await place(page, 'Table 2').selectOption({ label: 'M16 1' });
	await expect(page.getByTestId('team-places')).toHaveText('M16 1: Referee 1, Table 2');
	const other = await place(page, 'Table 1')
		.locator('option')
		.filter({ hasNotText: /^(—|M16 1)$/ })
		.first()
		.textContent();
	await place(page, 'Table 1').selectOption({ label: other! });
	await expect(place(page, 'Table 1')).not.toHaveValue('');
	await expect(page.getByTestId('team-places')).toHaveText('M16 1: Referee 1, Table 2');

	// "—" empties a place.
	await place(page, 'Referee 1').selectOption('');
	await expect(page.getByTestId('team-places')).toHaveText('M16 1: Table 2');

	// "Schedule entered", also undone with a second tap.
	const entered = page.getByRole('button', { name: 'Schedule entered' });
	await entered.click();
	await expect(page.getByTestId('schedule-state')).toHaveText('Schedule entered');
	await entered.click();
	await expect(page.getByTestId('schedule-state')).toHaveText('Schedule not entered');
	await entered.click();
	await expect(page.getByTestId('schedule-state')).toHaveText('Schedule entered');

	// After a reload everything is stored.
	await page.reload();
	await expect(place(page, 'Referee 1')).toHaveValue('');
	await expect(place(page, 'Table 1').locator('option:checked')).toHaveText(other!);
	await expect(page.getByTestId('team-places')).toHaveText('M16 1: Table 2');
	await expect(page.getByTestId('schedule-state')).toHaveText('Schedule entered');
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([]);
});

test('an away game proposes drivers until a number is set', async ({ page }) => {
	const errors = watchConsole(page);
	const game = await findGame(page, awayGame);
	const proposal = game.places!.drivers!.proposal;
	const other = proposal === 7 ? 6 : 7;
	await page.goto(`/game/2026-2027/${game.id}`);
	const drivers = page.getByTestId('drivers');
	const field = page.getByLabel('Number of drivers');
	const save = page.getByRole('button', { name: 'Save' });
	await expect(drivers).toHaveText(`Drivers: ${proposal} (proposed)`);
	await expect(page.getByRole('button', { name: 'Use proposal' })).toHaveCount(0);

	// The proposal saved unchanged stays the proposal.
	const saved = page.waitForResponse('**/drivers');
	await save.click();
	expect((await (await saved).json()).game.places.drivers.proposed).toBe(true);
	await expect(drivers).toHaveText(`Drivers: ${proposal} (proposed)`);

	// A set number stays, also after a reload.
	await field.fill(String(other));
	await save.click();
	await expect(drivers).toHaveText(`Drivers: ${other}`);
	await expect(page.getByTestId('drivers-proposal')).toHaveText(
		`Proposal: ${proposal} (one for every 3 players)`
	);
	await page.reload();
	await expect(drivers).toHaveText(`Drivers: ${other}`);
	await expect(field).toHaveValue(String(other));

	// "Use proposal" goes back.
	await page.getByRole('button', { name: 'Use proposal' }).click();
	await expect(drivers).toHaveText(`Drivers: ${proposal} (proposed)`);
	await expect(page.getByRole('button', { name: 'Use proposal' })).toHaveCount(0);
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([]);
});

test('a number of drivers that is empty or not 0 to 10 cannot be saved', async ({ page }) => {
	const game = await findGame(page, awayGame);
	await page.goto(`/game/2026-2027/${game.id}`);
	const field = page.getByLabel('Number of drivers');
	const save = page.getByRole('button', { name: 'Save' });
	for (const value of ['', '2.5', '11', '-1', 'two']) {
		await field.fill(value);
		await expect(save, `"${value}"`).toBeDisabled();
	}
	await field.fill('10');
	await expect(save).toBeEnabled();
});

test('a save that fails shows "Not saved. Try again." and the stored places', async ({ page }) => {
	const game = await findGame(page, otherHomeGame);
	await page.route('**/api/games/*/*/places/**', (route) =>
		route.fulfill({ status: 500, json: { error: 'internal' } })
	);
	await page.goto(`/game/2026-2027/${game.id}`);
	await place(page, 'Table 2').selectOption('');
	await expect(page.getByTestId('not-saved')).toHaveText('Not saved. Try again.');
	// The stored place of the first test.
	await expect(place(page, 'Table 2').locator('option:checked')).toHaveText('M16 1');
	await expect(page.getByTestId('team-places')).toHaveText('M16 1: Table 2');
});

test('a game that changed since its places were chosen says so', async ({ page }) => {
	const home = await findGame(page, otherHomeGame);
	const away = await findGame(page, awayGame);
	await page.route('**/api/games/*/*', async (route) => {
		const answer = await (await route.fetch()).json();
		answer.game.in_list = false;
		if (answer.game.places?.schedule) answer.game.places.schedule.required = false;
		if (answer.game.places?.drivers) answer.game.places.drivers.required = false;
		await route.fulfill({ json: answer });
	});
	await page.goto(`/game/2026-2027/${home.id}`);
	await expect(page.getByTestId('places-warning')).toHaveText([
		'This game is no longer a home game of the club.',
		'no longer in the game list'
	]);
	await page.goto(`/game/2026-2027/${away.id}`);
	await expect(page.getByTestId('places-warning')).toHaveText([
		'This game no longer needs drivers of the team.',
		'no longer in the game list'
	]);
});

test('"Back" returns to the page the game was opened from', async ({ page }) => {
	const game = await findGame(page, otherHomeGame);
	await page.goto(`/game/2026-2027/${game.id}`);
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/season\?season=2026-2027$/);
	await page.goto(`/game/2026-2027/${game.id}?from=start`);
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/$/);
	await page.goto(`/game/2026-2027/${game.id}?from=changes`);
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/changes$/);
});

test('a game that cannot be found says so', async ({ page }) => {
	await page.goto('/game/2026-2027/1');
	await expect(page.getByTestId('game-missing')).toHaveText('This game cannot be found.');
	await page.getByRole('link', { name: 'Season' }).click();
	await expect(page).toHaveURL(/\/season$/);
});

const label = (page: Page, name: string) =>
	page.getByTestId('labels').getByRole('button', { name, exact: true });
const row = (page: Page, id: number | string) =>
	page.locator(`[data-testid="game"][data-id="${id}"]`);

test('the rows of "Season" show the places of the team and "Schedule not entered"', async ({
	page
}) => {
	const homes = (await comingGames(page)).filter(otherHomeGame);
	// The game of the first test: entered, Table 2 of the team.
	await page.goto('/season');
	await expect(row(page, homes[0].id).getByTestId('game-places')).toHaveText('Table 2: open');
	// A game with no place chosen.
	await expect(row(page, homes[2].id).getByTestId('game-places')).toHaveText(
		'Schedule not entered'
	);

	// Places of the team chosen, the schedule not entered (S18): both show.
	await page.goto(`/game/2026-2027/${homes[1].id}`);
	const chosen: string[] = [];
	for (const name of ['Referee 1', 'Table 1', 'Table 2']) {
		await place(page, name).selectOption({ label: 'M16 1' });
		chosen.push(name);
		// The answer of the save, not only the choice on the screen.
		await expect(page.getByTestId('team-places')).toHaveText(`M16 1: ${chosen.join(', ')}`);
	}
	await page.goto('/season');
	await expect(row(page, homes[1].id).getByTestId('game-places')).toHaveText(
		'Referee 1: open · Table 1, 2: open, open · Schedule not entered'
	);
	// An away game of the team shows its drivers.
	const away = await findGame(page, awayGame);
	await expect(row(page, away.id).getByTestId('game-places')).toHaveText(
		/^Drivers \(\d+, proposed\): \d+ open$/
	);
	await expectAccessible(page);
});

test('the walk through "Not entered" returns with the labels and without the game', async ({
	page
}) => {
	const errors = watchConsole(page);
	await page.goto('/season');
	await label(page, 'Not entered').click();
	const rows = page.getByTestId('part-coming').getByTestId('game');
	const before = await rows.count();
	expect(before).toBeGreaterThan(2);
	await expect(page.getByTestId('part-coming').getByTestId('training-row')).toHaveCount(0);
	for (const text of await rows.getByTestId('game-places').allTextContents())
		expect(text).toMatch(/Schedule not entered$/);

	// A tap on a game lower in the list opens it.
	const game = rows.nth(2);
	const id = await game.getAttribute('data-id');
	await game.scrollIntoViewIfNeeded();
	const scrolled = await page.evaluate(() => window.scrollY);
	await game.getByTestId('game-link').click();
	await expect(page).toHaveURL(new RegExp(`/game/2026-2027/${id}$`));
	await place(page, 'Referee 2').selectOption({ label: 'M16 1' });
	await expect(page.getByTestId('team-places')).toHaveText('M16 1: Referee 2');
	await page.getByRole('button', { name: 'Schedule entered' }).click();
	await expect(page.getByTestId('schedule-state')).toHaveText('Schedule entered');

	// "Back": the same labels and place, without that game.
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/season\?season=2026-2027$/);
	await expect(label(page, 'Not entered')).toHaveAttribute('aria-pressed', 'true');
	await expect(label(page, 'Coming')).toHaveAttribute('aria-pressed', 'true');
	await expect(rows).toHaveCount(before - 1);
	await expect(row(page, id!)).toHaveCount(0);
	if (scrolled > 0) expect(await page.evaluate(() => window.scrollY)).toBeGreaterThan(0);

	// The browser's back from a game keeps them too.
	await rows.first().getByTestId('game-link').click();
	await expect(page).toHaveURL(/\/game\//);
	await page.goBack();
	await expect(label(page, 'Not entered')).toHaveAttribute('aria-pressed', 'true');
	expect(errors).toEqual([]);
});

test('"Entered" and "Open places" choose the games with those duties', async ({ page }) => {
	const homes = (await comingGames(page)).filter(otherHomeGame);
	const away = await findGame(page, awayGame);
	await page.goto('/season');
	const coming = page.getByTestId('part-coming');
	const rows = coming.getByTestId('game');

	await label(page, 'Entered').click();
	await expect(row(page, homes[0].id)).toBeVisible();
	await expect(row(page, homes[1].id)).toHaveCount(0);
	await expect(row(page, away.id)).toHaveCount(0);
	await expect(coming.getByTestId('training-row')).toHaveCount(0);
	for (const text of await rows.getByTestId('game-places').allTextContents())
		expect(text).not.toMatch(/Schedule not entered/);

	await label(page, 'Entered').click();
	await label(page, 'Open places').click();
	await expect(row(page, homes[1].id)).toBeVisible();
	await expect(row(page, away.id)).toBeVisible();
	const none = (await comingGames(page)).find(
		(game) => otherHomeGame(game) && (game.places as { open_places: number }).open_places === 0
	);
	expect(none, 'a home game without places of the team').toBeTruthy();
	await expect(row(page, none!.id)).toHaveCount(0);
	for (const text of await rows.getByTestId('game-places').allTextContents())
		expect(text).toMatch(/open/);

	// With "Games of the team" as well, only those games show.
	await label(page, 'Games of the team').click();
	await expect(row(page, homes[1].id)).toHaveCount(0);
	await expect(row(page, away.id)).toBeVisible();
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
});

type GameEvent = { date: string; game?: Game & { date: string; start_time?: string | null } };

test('the open item of more home games opens their list, and a tap opens a game', async ({
	page
}) => {
	const games = (await comingGames(page)).filter(otherHomeGame).slice(0, 2) as (Game & {
		date: string;
	})[];
	const item = (list: typeof games) => ({
		kind: 'schedule_not_entered',
		games: list.map((game) => ({
			season: '2026-2027',
			id: game.id,
			date: game.date,
			start_time: null,
			home: game.home,
			away: game.away
		}))
	});
	let items = [item(games)];
	await page.route('**/api/start', async (route) => {
		const start = await (await route.fetch()).json();
		start.open_items = items;
		await route.fulfill({ json: start });
	});
	await page.goto('/');
	const open = page.locator('[data-testid="open-item"][data-kind="schedule_not_entered"]');
	await expect(open).toHaveText('2 home games: schedule not entered');
	await expect(open).toHaveAttribute('aria-expanded', 'false');
	await open.click();
	await expect(open).toHaveAttribute('aria-expanded', 'true');
	const list = page.getByTestId('open-item-game');
	await expect(list).toHaveCount(2);
	await expect(list.first()).toContainText(
		`time not known yet ${games[0].home} – ${games[0].away}`
	);
	await expectAccessible(page);
	await list.first().click();
	await expect(page).toHaveURL(new RegExp(`/game/2026-2027/${games[0].id}\\?from=start$`));
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/$/);

	// One game: the item opens the game itself.
	items = [item(games.slice(1))];
	await page.reload();
	await expect(open).toHaveText(/^Home game \w{3} \d+ \w{3}: schedule not entered$/);
	await open.click();
	await expect(page).toHaveURL(new RegExp(`/game/2026-2027/${games[1].id}\\?from=start$`));
});

test('"Next" shows a home game of another club team with places of the team', async ({ page }) => {
	// The game of the first test has Table 2 of the team; it is after the two weeks of 12 October,
	// so the answer moves it in.
	const game = (await comingGames(page)).find(otherHomeGame)!;
	const season = await (await page.request.get('/api/season')).json();
	const event = season.coming.find((e: GameEvent) => e.game?.id === game.id);
	await page.route('**/api/start', async (route) => {
		const start = await (await route.fetch()).json();
		start.next = [...start.next, event];
		await route.fulfill({ json: start });
	});
	await page.goto('/');
	const next = page.getByTestId('next').locator(`[data-testid="game"][data-id="${game.id}"]`);
	await expect(next.getByTestId('game-places')).toHaveText('Table 2: open');
	await next.getByTestId('game-link').click();
	await expect(page).toHaveURL(new RegExp(`/game/2026-2027/${game.id}\\?from=start$`));
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/$/);
});

test('"Last" shows the last games newest first, also without a training', async ({ page }) => {
	const season = await (await page.request.get('/api/season')).json();
	// The newest past game; the last training of the earlier tests is older.
	const event = season.past.find((e: GameEvent) => e.game);
	let withTraining = true;
	await page.route('**/api/start', async (route) => {
		const start = await (await route.fetch()).json();
		if (!withTraining) start.last = null;
		start.last_games = [event];
		start.last_games_season = '2026-2027';
		await route.fulfill({ json: start });
	});
	await page.goto('/');
	const last = page.getByTestId('last');
	const training = last.getByTestId('training-row');
	await expect(training).toHaveCount(1);
	const days = await last.getByTestId('day').allTextContents();
	expect(days.length).toBe(2);
	// The game's date first, the training's after it.
	const order = await last
		.locator('[data-testid="game"], [data-testid="training-row"]')
		.evaluateAll((rows) => rows.map((row) => row.getAttribute('data-testid')));
	expect(order).toEqual(['game', 'training-row']);

	withTraining = false;
	await page.reload();
	await expect(training).toHaveCount(0);
	const game = last.locator(`[data-testid="game"][data-id="${event.game.id}"]`);
	await expect(game).toBeVisible();
	await game.getByTestId('game-link').click();
	await expect(page).toHaveURL(new RegExp(`/game/2026-2027/${event.game.id}\\?from=start$`));
});

test('a change of a game with places of the team says "has duties", and a tap opens the game', async ({
	page
}) => {
	// The fake source never changes, so the answer of the list of changes is replaced: a move of
	// the game of the first test (Table 2 of the team), with its places as the server gives them.
	const errors = watchConsole(page);
	const season = await (await page.request.get('/api/season')).json();
	const event = season.coming.find((e: GameEvent) => e.game && otherHomeGame(e.game));
	const { id, places, date, start_time, home, away, hall, town } = event.game;
	const game = { date, start_time, home, away, hall, town };
	await page.route('**/api/changes', (route) =>
		route.fulfill({
			json: {
				season: '2026-2027',
				changes: [
					{
						id,
						version: 2,
						kind: 'moved',
						differences: ['place'],
						reason: 'duty',
						before: { ...game, hall: 'Elsewhere' },
						after: game,
						places
					}
				]
			}
		})
	);
	await page.goto('/changes');
	const change = page.getByTestId('change');
	await expect(change.getByTestId('change-duties')).toHaveText('has duties · Table 2: open');
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	await change.getByTestId('change-game').click();
	await expect(page).toHaveURL(new RegExp(`/game/2026-2027/${id}\\?from=changes$`));
	await expect(page.getByTestId('team-places')).toHaveText('M16 1: Table 2');
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/changes$/);
	expect(errors).toEqual([]);
});
