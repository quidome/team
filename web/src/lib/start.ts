// The start page (story B of start page and season): the words and the links of the open items.
import { resolve } from '$app/paths';
import type { DriverGameView, OpenGame, OpenItemView, TooFewGameView } from '#lib/api/client.js';
import { formatDay } from '#lib/schedule.js';

/** "1 change not seen", "3 changes not seen". */
export function unseenLine(count: number): string {
	return count === 1 ? '1 change not seen' : `${count} changes not seen`;
}

/** The line of an open item, for example "Training Thu 22 Oct not checked". */
export function openItemText(item: OpenItemView): string {
	switch (item.kind) {
		case 'reading_held_back':
			return 'The last reading of the game list was held back.';
		case 'out_of_date':
			return `The games may be out of date since ${formatDay(item.since)}.`;
		case 'changes':
			return unseenLine(item.count);
		case 'too_few_players': {
			const [first] = item.games;
			return item.games.length === 1 && first
				? tooFewText(first)
				: `${item.games.length} games: too few players`;
		}
		case 'driver_not_coming': {
			const [first] = item.games;
			return item.games.length === 1 && first
				? driverText(first)
				: `${item.games.length} games: a driver is not coming`;
		}
		case 'answers_moved': {
			const [first] = item.games;
			return item.games.length === 1 && first
				? `Game ${formatDay(first.date)} moved: check the answers`
				: `${item.games.length} games moved: check the answers`;
		}
		case 'schedule_not_entered': {
			const [first] = item.games;
			return item.games.length === 1 && first
				? `Home game ${formatDay(first.date)}: schedule not entered`
				: `${item.games.length} home games: schedule not entered`;
		}
		case 'open_places': {
			const [first] = item.games;
			return item.games.length === 1 && first
				? openGameText(first)
				: `${item.games.length} games with open places`;
		}
		case 'not_opened':
			return item.count === 1
				? `Training day ${formatDay(item.first)} not opened`
				: `${item.count} training days not opened`;
		case 'not_checked':
			return item.count === 1
				? `Training ${formatDay(item.first)} not checked`
				: `${item.count} trainings not checked`;
		case 'games_not_checked':
			return item.count === 1
				? `Game ${formatDay(item.first.date)} not checked`
				: `${item.count} games not checked`;
	}
}

/** Where a tap on an open item goes: the "Game list" card, the list of changes, the oldest
 * training day or training, the oldest game not checked, or the one game (with `?from=start`, so "Back" returns here). `null`
 * for an item of more games: a tap opens their list below it. */
export function openItemLink(item: OpenItemView): string | null {
	switch (item.kind) {
		case 'reading_held_back':
		case 'out_of_date':
			return `${resolve('/season')}#game-list`;
		case 'changes':
			return resolve('/changes');
		case 'too_few_players':
		case 'driver_not_coming':
		case 'answers_moved':
		case 'schedule_not_entered':
		case 'open_places': {
			const [first] = item.games;
			return item.games.length === 1 && first ? gameLink(first.season, first.id) : null;
		}
		case 'not_opened':
		case 'not_checked':
			return trainingLink(item.first);
		case 'games_not_checked':
			return gameLink(item.first.season, item.first.id);
	}
}

/** "Game Sat 31 Oct: 5 of 7 players, 3 no answer"; without "no answer": "Game Sat 31 Oct: 5 of 7
 * players". */
function tooFewText(game: TooFewGameView): string {
	const open = game.no_answer > 0 ? `, ${game.no_answer} no answer` : '';
	return `Game ${formatDay(game.date)}: ${game.coming} of ${game.enough} players${open}`;
}

/** "Away game Sat 31 Oct: driver Anna is not coming", "Away game Sat 31 Oct: 2 drivers are not
 * coming". */
function driverText(game: DriverGameView): string {
	const [name] = game.drivers;
	const who =
		game.drivers.length === 1 && name !== undefined
			? `driver ${name} is not coming`
			: `${game.drivers.length} drivers are not coming`;
	return `Away game ${formatDay(game.date)}: ${who}`;
}

/** "2 drivers open", "1 table place open", "1 referee place, 2 table places open". */
function openCount(count: number, one: string, more: string): string[] {
	return count === 0 ? [] : [`${count} ${count === 1 ? one : more}`];
}

/** The line of one game with open places: "Away game Sat 31 Oct: 2 drivers open", "Home game Sun
 * 1 Nov: 1 table place open". */
function openGameText(game: OpenGame): string {
	const where = game.drive > 0 ? 'Away game' : 'Home game';
	const parts = [
		...openCount(game.referee, 'referee place', 'referee places'),
		...openCount(game.table, 'table place', 'table places'),
		...openCount(game.drive, 'driver', 'drivers')
	];
	return `${where} ${formatDay(game.date)}: ${parts.join(', ')} open`;
}

/** The page of a game, opened from the start page: "Back" returns here. */
export function gameLink(season: string, id: number): string {
	return `${resolve('/game/[season]/[id]', { season, id: String(id) })}?from=start`;
}

/** A game of an open item: "Sat 31 Oct · 14:30 Blue Drakes X14 1 – BV Alphia X14 1". */
export function gameItemText(game: {
	date: string;
	start_time?: string | null;
	home: string;
	away: string;
}): string {
	return `${formatDay(game.date)} · ${game.start_time ?? 'time not known yet'} ${game.home} – ${game.away}`;
}

/** The training of `date`, opened from the start page: "Back" returns here. */
export function trainingLink(date: string): string {
	return `${resolve('/training/[date]', { date })}?from=start`;
}
