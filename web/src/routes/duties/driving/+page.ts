import { loadDrivingOverview } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	if (!me) return { message: null };
	return { message: await loadDrivingOverview(event.fetch) };
};
