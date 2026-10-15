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
