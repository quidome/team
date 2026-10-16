import { loadReasons, loadWeeklyTrainings } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	if (!me) return { weekly: null, reasons: null };
	const [weekly, reasons] = await Promise.all([
		loadWeeklyTrainings(event.fetch),
		loadReasons(event.fetch)
	]);
	return { weekly, reasons };
};
