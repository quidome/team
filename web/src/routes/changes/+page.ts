import { loadChanges } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	if (!me) return { changes: null, season: null };
	const answer = await loadChanges(event.fetch);
	return { changes: answer.changes, season: answer.season ?? null };
};
