// The filter of the list of changes (games of the team, possible duty games), and date words.
import type { ScheduleGame } from '#lib/api/client.js';

export type Filter = 'all' | 'team' | 'duty';

export const filters: { value: Filter; label: string }[] = [
	{ value: 'all', label: 'All' },
	{ value: 'team', label: 'Games of the team' },
	{ value: 'duty', label: 'Possible duty games' }
];

/** The games (or changes) that the filter shows, by their `reason`. */
export function filterGames<T extends Pick<ScheduleGame, 'reason'>>(
	games: T[],
	filter: Filter
): T[] {
	if (filter === 'all') return games;
	return games.filter((game) => game.reason === filter);
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
