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

// Story B: the players on the places of the team.

/** Writes of the API come from the tool's own origin. */
const origin = { headers: { Origin: 'http://localhost:4100' } };

/** Adds a player with the role "plays games" from the start of the season; returns the id. */
async function addPlayer(page: Page, first: string): Promise<number> {
	const response = await page.request.post('/api/players', {
		...origin,
		data: { first_name: first, last_name: null, role: { role: 'plays_games', from: '2026-09-01' } }
	});
	expect(response.ok(), await response.text()).toBe(true);
	return (await response.json()).player.id;
}

/** The place of the team `name` on the page of a game. */
const teamPlace = (page: Page, name: string) =>
	page.getByTestId('team-place').filter({ has: page.getByRole('button', { name }) });

const choicePlayer = (page: Page, name: string) =>
	page.getByTestId('choice-player').filter({ hasText: name });

test('a tap on a place of the team and a player puts the player on it; "Clear" opens it', async ({
	page
}) => {
	const errors = watchConsole(page);
	await addPlayer(page, 'Yara');
	await addPlayer(page, 'Yusuf');
	// The game of the first test, with Table 2 of the team.
	const game = await findGame(page, otherHomeGame);
	await page.goto(`/game/2026-2027/${game.id}`);
	const table2 = teamPlace(page, 'Table 2');
	await expect(table2.getByTestId('place-player')).toHaveText('open');

	await table2.getByRole('button', { name: 'Table 2' }).click();
	await expect(choicePlayer(page, 'Yara')).toContainText('0 done · 0 planned');
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	await choicePlayer(page, 'Yara').click();
	await expect(table2.getByTestId('place-player')).toHaveText('Yara');
	await expect(page.getByTestId('choice')).toHaveCount(0);

	// The totals in the choice count the place; Yara is on it, so not offered again.
	await table2.getByRole('button', { name: 'Table 2' }).click();
	await expect(choicePlayer(page, 'Yara')).toContainText('0 done · 1 planned');
	await expect(choicePlayer(page, 'Yara')).toContainText('on this place');
	await expect(choicePlayer(page, 'Yara')).toBeDisabled();

	// After a reload the player is stored; "Season" shows the name.
	await page.reload();
	await expect(table2.getByTestId('place-player')).toHaveText('Yara');
	const season = await (await page.request.get('/api/season')).json();
	const row = season.coming.find((e: GameEvent) => e.game?.id === game.id);
	expect(row.game.places.schedule.of_team[0].player.name).toBe('Yara');

	await table2.getByRole('button', { name: 'Table 2' }).click();
	await page.getByRole('button', { name: 'Clear' }).click();
	await expect(table2.getByTestId('place-player')).toHaveText('open');
	expect(errors).toEqual([]);
});

test('on a past game "Nobody" and "Someone else" correct a place', async ({ page }) => {
	const errors = watchConsole(page);
	const season = await (await page.request.get('/api/season')).json();
	const event = season.past.find((e: GameEvent) => e.game?.places?.schedule);
	expect(event, 'a past home game of the club').toBeTruthy();
	const id = event.game.id;
	const path = `/api/games/2026-2027/${id}`;
	const team = (await (await page.request.get(path)).json()).team.number;
	expect(
		(await page.request.put(`${path}/places/table/1`, { ...origin, data: { team } })).ok()
	).toBe(true);
	await page.goto(`/game/2026-2027/${id}`);
	const table1 = teamPlace(page, 'Table 1');
	await table1.getByRole('button', { name: 'Table 1' }).click();
	await choicePlayer(page, 'Yara').click();
	await expect(table1.getByTestId('place-player')).toHaveText('Yara');

	// A past place with a player first offers the corrections.
	await table1.getByRole('button', { name: 'Table 1' }).click();
	await page.getByRole('button', { name: 'Nobody' }).click();
	await expect(table1.getByTestId('place-player')).toHaveText('not done (was Yara)');

	await table1.getByRole('button', { name: 'Table 1' }).click();
	await page.getByRole('button', { name: 'Someone else' }).click();
	await choicePlayer(page, 'Yusuf').click();
	await expect(table1.getByTestId('place-player')).toHaveText('Yusuf (was Yara)');
	await table1.getByRole('button', { name: 'Table 1' }).click();
	await page.getByRole('button', { name: 'Someone else' }).click();
	await expect(choicePlayer(page, 'Yusuf')).toContainText('1 done · 0 planned');
	await expect(choicePlayer(page, 'Yara')).toContainText('0 done · 0 planned');
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);

	// Clean up for the later tests.
	await page.getByRole('button', { name: 'Clear' }).click();
	await expect(table1.getByTestId('place-player')).toHaveText('open');
	expect(
		(await page.request.put(`${path}/places/table/1`, { ...origin, data: { team: null } })).ok()
	).toBe(true);
	expect(errors).toEqual([]);
});

test('a blocked player is shown with the reason and cannot be chosen', async ({ page }) => {
	const game = await findGame(page, otherHomeGame);
	await page.route(`**/api/games/2026-2027/${game.id}`, async (route) => {
		const answer = await (await route.fetch()).json();
		for (const player of answer.players)
			if (player.name === 'Yara') {
				player.clashes = [{ level: 'blocked', duty: 'referee', start: '14:00' }];
				for (const offer of player.offers) offer.assign = 'clash';
			} else if (player.name === 'Yusuf')
				player.clashes = [{ level: 'warning', duty: null, start: '15:00' }];
		await route.fulfill({ json: answer });
	});
	await page.goto(`/game/2026-2027/${game.id}`);
	await page.getByRole('button', { name: 'Table 2' }).click();
	const yara = choicePlayer(page, 'Yara');
	await expect(yara).toContainText('at another game at almost the same time');
	await expect(yara).toContainText('also Referee 14:00');
	await expect(yara).toBeDisabled();
	const yusuf = choicePlayer(page, 'Yusuf');
	await expect(yusuf).toContainText('plays 15:00');
	await expect(yusuf).toBeEnabled();
});

test('a choice that fails shows "Not saved. Try again." and the stored place', async ({ page }) => {
	const game = await findGame(page, otherHomeGame);
	await page.route('**/api/games/*/*/assignments/**', (route) =>
		route.fulfill({ status: 500, json: { error: 'internal' } })
	);
	await page.goto(`/game/2026-2027/${game.id}`);
	const table2 = teamPlace(page, 'Table 2');
	await table2.getByRole('button', { name: 'Table 2' }).click();
	await choicePlayer(page, 'Yusuf').click();
	await expect(page.getByTestId('not-saved')).toHaveText('Not saved. Try again.');
	await expect(table2.getByTestId('place-player')).toHaveText('open');
});

test('the open item "games with open places" names one game and lists more', async ({ page }) => {
	const errors = watchConsole(page);
	// The game of the first test has Table 2 of the team without a player.
	const game = await findGame(page, otherHomeGame);
	const real = (await (await page.request.get('/api/start')).json()).open_items.find(
		(item: { kind: string }) => item.kind === 'open_places'
	);
	expect(real.games.map((open: { id: number }) => open.id)).toContain(game.id);

	const base = { season: '2026-2027', start_time: '14:30', home: 'Home', away: 'Away' };
	const away = { ...base, id: game.id, date: '2026-10-31', referee: 0, table: 0, drive: 2 };
	const home = { ...base, id: game.id, date: '2026-11-01', referee: 1, table: 2, drive: 0 };
	let games = [away];
	await page.route('**/api/start', async (route) => {
		const start = await (await route.fetch()).json();
		start.open_items = [{ kind: 'open_places', games }];
		await route.fulfill({ json: start });
	});
	await page.goto('/');
	const open = page.locator('[data-testid="open-item"][data-kind="open_places"]');
	await expect(open).toHaveText('Away game Sat 31 Oct: 2 drivers open');
	games = [{ ...home, referee: 0, table: 1 }];
	await page.reload();
	await expect(open).toHaveText('Home game Sun 1 Nov: 1 table place open');
	games = [home];
	await page.reload();
	await expect(open).toHaveText('Home game Sun 1 Nov: 1 referee place, 2 table places open');
	await open.click();
	await expect(page).toHaveURL(new RegExp(`/game/2026-2027/${game.id}\\?from=start$`));

	// More games: a tap opens their list.
	games = [away, { ...home, id: game.id + 1 }, { ...home, id: game.id + 2 }];
	await page.goto('/');
	await expect(open).toHaveText('3 games with open places');
	await open.click();
	const list = page.getByTestId('open-item-game');
	await expect(list).toHaveCount(3);
	await expect(list.first()).toHaveText('Sat 31 Oct · 14:30 Home – Away');
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	await list.first().click();
	await expect(page).toHaveURL(new RegExp(`/game/2026-2027/${game.id}\\?from=start$`));
	expect(errors).toEqual([]);
});

test('"Duties" shows the totals of each player in a season', async ({ page }) => {
	const errors = watchConsole(page);
	// The earlier tests cleared their players again.
	await page.goto('/duties');
	await expect(page.getByRole('heading', { level: 1 })).toHaveText('Duties · 2026-2027');
	await expect(page.getByTestId('no-duties-yet')).toHaveText('No duties yet this season.');

	// Yara on Table 2 of the game of the first test: planned.
	const game = await findGame(page, otherHomeGame);
	const path = `/api/games/2026-2027/${game.id}/assignments/table/2`;
	const players = (await (await page.request.get(`/api/games/2026-2027/${game.id}`)).json())
		.players as { id: number; name: string }[];
	const yara = players.find((player) => player.name === 'Yara')!;
	expect((await page.request.put(path, { ...origin, data: { player: yara.id } })).ok()).toBe(true);
	await page.reload();
	const row = page.getByTestId('duty-row').filter({ hasText: 'Yara' });
	await expect(row.locator('td')).toHaveText(['0 · 0', '0 · 1', '0 · 0']);
	await expect(page.getByTestId('duty-row').filter({ hasText: 'Yusuf' }).locator('td')).toHaveText([
		'0 · 0',
		'0 · 0',
		'0 · 0'
	]);
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);

	// Another season, from the address; a season that cannot be read.
	await page.goto('/duties?season=2025-2026');
	await expect(page.getByRole('heading', { level: 1 })).toHaveText('Duties · 2025-2026');
	await expect(page.getByTestId('no-duties-yet')).toBeVisible();

	// The link on the start page.
	await page.goto('/');
	await page.getByTestId('team-links').getByRole('link', { name: 'Duties' }).click();
	await expect(page).toHaveURL(/\/duties$/);
	expect((await page.request.delete(path, origin)).ok()).toBe(true);
	expect(errors).toEqual([]);

	// The answer 400 of this address is expected, so the console is checked before it.
	await page.goto('/duties?season=2026');
	await expect(page.getByTestId('invalid-season')).toHaveText('This season cannot be read.');
});

test('"Delete player" is not offered for a player with a duty', async ({ page }) => {
	const errors = watchConsole(page);
	const game = await findGame(page, otherHomeGame);
	const path = `/api/games/2026-2027/${game.id}/assignments/table/2`;
	const players = (await (await page.request.get(`/api/games/2026-2027/${game.id}`)).json())
		.players as { id: number; name: string }[];
	const yusuf = players.find((player) => player.name === 'Yusuf')!;
	await page.goto(`/players/${yusuf.id}`);
	await expect(page.getByRole('button', { name: 'Delete player' })).toBeVisible();

	expect((await page.request.put(path, { ...origin, data: { player: yusuf.id } })).ok()).toBe(true);
	await page.reload();
	await expect(page.getByTestId('player-name')).toHaveText('Yusuf');
	await expect(page.getByRole('button', { name: 'Delete player' })).toHaveCount(0);
	// The server refuses it too, with its own code.
	const refused = await page.request.delete(`/api/players/${yusuf.id}`, origin);
	expect(refused.status()).toBe(400);
	expect((await refused.json()).error).toBe('player_has_duties');
	expect((await page.request.delete(path, origin)).ok()).toBe(true);
	expect(errors).toEqual([]);
});

test('under a corrected place the note is of the player who did it', async ({ page }) => {
	const game = await findGame(page, otherHomeGame);
	await page.route(`**/api/games/2026-2027/${game.id}`, async (route) => {
		const answer = await (await route.fetch()).json();
		const yara = answer.players.find((player: { name: string }) => player.name === 'Yara');
		const yusuf = answer.players.find((player: { name: string }) => player.name === 'Yusuf');
		yara.clashes = [{ level: 'warning', duty: null, start: '10:00' }];
		yusuf.clashes = [{ level: 'warning', duty: 'table', start: '13:00' }];
		answer.game.places.schedule.of_team[0].player = {
			id: yara.id,
			name: 'Yara',
			not_in_team: false,
			outcome: 'done_by',
			done_by: { id: yusuf.id, name: 'Yusuf' }
		};
		await route.fulfill({ json: answer });
	});
	await page.goto(`/game/2026-2027/${game.id}`);
	const table2 = teamPlace(page, 'Table 2');
	await expect(table2.getByTestId('place-player')).toHaveText('Yusuf (was Yara)');
	await expect(table2.getByTestId('place-clash')).toHaveText(['also Table 13:00']);
});
