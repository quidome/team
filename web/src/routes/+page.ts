import {
	loadLatestReading,
	loadNextTraining,
	loadPageSelection,
	loadSchedule
} from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { selection } = await loadPageSelection(event);
	// With a team needed, the schedule is last season's (or none).
	const state = selection?.state;
	// The next training does not need a chosen team.
	const [readings, schedule, nextTraining] = await Promise.all([
		state === 'chosen' ? loadLatestReading(event.fetch) : null,
		state === 'chosen' || state === 'team_needed' ? loadSchedule(event.fetch) : null,
		selection ? loadNextTraining(event.fetch) : null
	]);
	return { selection, readings, schedule, nextTraining };
};
