import { ApiError, loadAttendance, loadTrainings } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	if (!me) return { list: null, overview: null, invalidSeason: false };
	// The chosen season is in the address, so a reload and "Back" from a training keep it.
	const season = event.url.searchParams.get('season') ?? undefined;
	try {
		const [list, overview] = await Promise.all([
			loadTrainings(event.fetch, season),
			loadAttendance(event.fetch, season)
		]);
		return { list, overview, invalidSeason: false };
	} catch (failure) {
		if (failure instanceof ApiError && failure.code === 'invalid_season')
			return { list: null, overview: null, invalidSeason: true };
		throw failure;
	}
};
