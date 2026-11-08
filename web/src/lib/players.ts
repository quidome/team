// Words for players, roles and training days (attendance at training, story A).
import type { RoleKind } from '#lib/api/client.js';
import { formatDay } from '#lib/schedule.js';

export const roleLabels: Record<RoleKind, string> = {
	plays_games: 'Plays games',
	only_trains: 'Only trains'
};

/** "Avery Jansen", or "Casey" without a last name. */
export function fullName(player: { first_name: string; last_name?: string | null }): string {
	return player.last_name ? `${player.first_name} ${player.last_name}` : player.first_name;
}

/** The day after a date such as "2026-10-15": "2026-10-16". */
export function dayAfter(date: string): string {
	const next = new Date(`${date}T00:00:00Z`);
	next.setUTCDate(next.getUTCDate() + 1);
	return next.toISOString().slice(0, 10);
}

/** The days of the week, 1 = Monday, as the API numbers them. */
export const weekdays: { value: number; label: string }[] = [
	{ value: 1, label: 'Monday' },
	{ value: 2, label: 'Tuesday' },
	{ value: 3, label: 'Wednesday' },
	{ value: 4, label: 'Thursday' },
	{ value: 5, label: 'Friday' },
	{ value: 6, label: 'Saturday' },
	{ value: 7, label: 'Sunday' }
];

/** "This player now counts as there at 2 earlier trainings: Tue 1 Sept, Tue 8 Sept." With a name
 * in place of "This player" when more than one player is named. */
export function nowThereText(trainings: string[], name?: string): string {
	const count = trainings.length;
	const what = count === 1 ? '1 earlier training' : `${count} earlier trainings`;
	return `${name ?? 'This player'} now counts as there at ${what}: ${trainings.map(formatDay).join(', ')}.`;
}

/** "This player has no answer at 2 checked games: Sat 10 Oct, Sat 17 Oct." With a name in place of
 * "This player" when more than one player is named. */
export function noAnswerText(games: string[], name?: string): string {
	const count = games.length;
	const what = count === 1 ? '1 checked game' : `${count} checked games`;
	return `${name ?? 'This player'} has no answer at ${what}: ${games.map(formatDay).join(', ')}.`;
}

/** The notes after a change of role: the trainings at which the player now counts as there, and
 * the checked games at which the player has no answer; each only when it has dates. */
export function nowThereLines(
	entry: { trainings: string[]; games: string[] },
	name?: string
): string[] {
	return [
		...(entry.trainings.length > 0 ? [nowThereText(entry.trainings, name)] : []),
		...(entry.games.length > 0 ? [noAnswerText(entry.games, name)] : [])
	];
}
