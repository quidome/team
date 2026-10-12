// The schedule on the start page (story 3b): filter, grouping by date, and date words.
import type { ScheduleGame } from '#lib/api/client.js';

export type Filter = 'all' | 'team' | 'duty';

export const filters: { value: Filter; label: string }[] = [
	{ value: 'all', label: 'All' },
	{ value: 'team', label: 'Games of the team' },
	{ value: 'duty', label: 'Possible duty games' }
];

export function filterGames(games: ScheduleGame[], filter: Filter): ScheduleGame[] {
	if (filter === 'all') return games;
	return games.filter((game) => game.reason === filter);
}

/** Groups games by date and keeps their order (the server sorts them). */
export function groupByDate(games: ScheduleGame[]): { date: string; games: ScheduleGame[] }[] {
	const groups: { date: string; games: ScheduleGame[] }[] = [];
	for (const game of games) {
		const last = groups.at(-1);
		if (last?.date === game.date) last.games.push(game);
		else groups.push({ date: game.date, games: [game] });
	}
	return groups;
}

/** "Sat 31 Oct" for a Dutch date such as "2026-10-31". */
export function formatDay(date: string): string {
	return new Intl.DateTimeFormat('en-GB', {
		weekday: 'short',
		day: 'numeric',
		month: 'short',
		timeZone: 'Europe/Amsterdam'
	}).format(new Date(`${date}T12:00:00Z`));
}
