import { ApiError, loadLatestReading, loadSeason } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	if (!me) return { answer: null, readings: null, invalidSeason: false };
	// The chosen season is in the address, so a reload and "Back" from a training keep it.
	const season = event.url.searchParams.get('season') ?? undefined;
	try {
		const answer = await loadSeason(event.fetch, season);
		// The "Game list" card belongs to the current season with a chosen team.
		const readings =
			answer.season === answer.current_season && answer.games === 'shown'
				? await loadLatestReading(event.fetch)
				: null;
		return { answer, readings, invalidSeason: false };
	} catch (failure) {
		if (failure instanceof ApiError && failure.code === 'invalid_season')
			return { answer: null, readings: null, invalidSeason: true };
		throw failure;
	}
};
