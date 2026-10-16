import { loadTeam } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	return { team: me ? await loadTeam(event.fetch) : null };
};
