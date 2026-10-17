import { loadTraining } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	return { day: me ? await loadTraining(event.fetch, event.params.date) : null };
};
