import { loadPlayer } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	if (!me) return { loggedIn: false, player: null };
	return { loggedIn: true, player: await loadPlayer(event.fetch, Number(event.params.id)) };
};
