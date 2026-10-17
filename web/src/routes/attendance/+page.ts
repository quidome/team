import { loadTrainings } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	return { list: me ? await loadTrainings(event.fetch) : null };
};
