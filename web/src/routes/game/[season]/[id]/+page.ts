import { ApiError, loadGame } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	if (!me) return { page: null, missing: false };
	try {
		return {
			page: await loadGame(event.fetch, event.params.season, event.params.id),
			missing: false
		};
	} catch (failure) {
		// A game that is not a game of the club chosen for its season, or an address that cannot
		// be read.
		if (failure instanceof ApiError) return { page: null, missing: true };
		throw failure;
	}
};
