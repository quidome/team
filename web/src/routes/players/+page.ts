import { loadPlayers } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	return { players: me ? await loadPlayers(event.fetch) : null };
};
