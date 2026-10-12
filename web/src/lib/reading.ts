// The words for a reading of the game list (story 3a).
import type { Reading, ReadingCounts } from '#lib/api/client.js';

/** The kinds of change in the order of the result line, with their words. */
const kinds: [keyof ReadingCounts, string][] = [
	['new', 'new'],
	['moved', 'moved'],
	['start_time_known', 'start time known'],
	['home_and_away', 'home and away swapped'],
	['team_changed', 'team changed'],
	['gone', 'no longer in the game list'],
	['came_back', 'came back'],
	['into_schedule', 'now in the schedule'],
	['out_of_schedule', 'no longer in the schedule']
];

/** The short result line of a good reading, for example "3 new, 1 moved". */
export function resultLine(reading: Reading): string {
	if (reading.first_reading || !reading.counts) return `${reading.games_read ?? 0} games read`;
	const counts = reading.counts;
	const parts = kinds
		.filter(([kind]) => counts[kind] > 0)
		.map(([kind, words]) => `${counts[kind]} ${words}`);
	return parts.length > 0 ? parts.join(', ') : 'No changes';
}

const dutch = new Intl.DateTimeFormat('en-GB', {
	timeZone: 'Europe/Amsterdam',
	weekday: 'short',
	day: 'numeric',
	month: 'long',
	year: 'numeric',
	hour: '2-digit',
	minute: '2-digit',
	hour12: false
});

/** A moment as Dutch local time, for example "Mon 12 October 2026 14:00". */
export function dutchTime(iso: string): string {
	const parts = Object.fromEntries(
		dutch.formatToParts(new Date(iso)).map((part) => [part.type, part.value])
	);
	return `${parts.weekday} ${parts.day} ${parts.month} ${parts.year} ${parts.hour}:${parts.minute}`;
}
