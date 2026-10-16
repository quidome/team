// Words for players, roles and training days (attendance at training, story A).
import type { Player, RoleKind } from '#lib/api/client.js';

export const roleLabels: Record<RoleKind, string> = {
	plays_games: 'Plays games',
	only_trains: 'Only trains'
};

/** "Avery Jansen", or "Casey" without a last name. */
export function fullName(player: Pick<Player, 'first_name' | 'last_name'>): string {
	return player.last_name ? `${player.first_name} ${player.last_name}` : player.first_name;
}

/** The day after a date such as "2026-10-15": "2026-10-16". */
export function dayAfter(date: string): string {
	const next = new Date(`${date}T00:00:00Z`);
	next.setUTCDate(next.getUTCDate() + 1);
	return next.toISOString().slice(0, 10);
}

/** The later of two dates such as "2026-10-15". */
export function later(first: string, second: string): string {
	return first > second ? first : second;
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
