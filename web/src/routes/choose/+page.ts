import { loadPageSelection } from '#lib/api/client.js';
import type { PageLoad } from './$types';

export const load: PageLoad = (event) => loadPageSelection(event);
