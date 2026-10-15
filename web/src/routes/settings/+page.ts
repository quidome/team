import { loadReasons, loadTrainingSettings } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	if (!me) return { settings: null, reasons: null };
	const [settings, reasons] = await Promise.all([
		loadTrainingSettings(event.fetch),
		loadReasons(event.fetch)
	]);
	return { settings, reasons };
};
