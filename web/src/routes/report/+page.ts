import { loadReport, type Report } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { me } = await event.parent();
	if (!me) return { report: null, loadError: false };
	let report: Report | null = null;
	let loadError = false;
	try {
		report = await loadReport(event.fetch);
	} catch {
		loadError = true;
	}
	return { report, loadError };
};
