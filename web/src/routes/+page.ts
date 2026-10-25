import { loadPageSelection, loadStart, type StartAnswer } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { selection } = await loadPageSelection(event);
	// With a club chosen (a team chosen, or still needed in a new season): the start page.
	const state = selection?.state;
	let start: StartAnswer | null = null;
	let startError = false;
	if (state === 'chosen' || state === 'team_needed') {
		try {
			start = await loadStart(event.fetch);
		} catch {
			startError = true;
		}
	}
	return { selection, start, startError };
};
