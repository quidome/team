import { loadLatestReading, loadPageSelection } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async (event) => {
	const { selection } = await loadPageSelection(event);
	const readings = selection?.state === 'chosen' ? await loadLatestReading(event.fetch) : null;
	return { selection, readings };
};
