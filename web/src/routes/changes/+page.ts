import { loadChanges } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	return { changes: me ? await loadChanges(event.fetch) : null };
};
