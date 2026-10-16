import { loadPlayers } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	if (!me) return { players: null, player: null };
	const players = await loadPlayers(event.fetch);
	const id = Number(event.params.id);
	// The page shows only roles of the current season; a player of last season has none yet.
	const player =
		[
			...players.players,
			...players.left,
			...players.last_season.map((candidate) => ({ ...candidate, roles: [] })),
			...players.others
		].find((candidate) => candidate.id === id) ?? null;
	return { players, player };
};
