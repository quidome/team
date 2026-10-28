import { ApiError, loadDuties } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	if (!me) return { overview: null, invalidSeason: false };
	// The chosen season is in the address, so a reload keeps it.
	const season = event.url.searchParams.get('season') ?? undefined;
	try {
		return { overview: await loadDuties(event.fetch, season), invalidSeason: false };
	} catch (failure) {
		if (failure instanceof ApiError && failure.code === 'invalid_season')
			return { overview: null, invalidSeason: true };
		throw failure;
	}
};
