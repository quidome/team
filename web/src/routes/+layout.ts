// The app runs in the browser only; the Rust server serves the built files (decision 0002).
import { loadMe } from '#lib/api/client.js';
import type { LayoutLoad } from './$types';

export const ssr = false;
export const prerender = false;

export const load: LayoutLoad = async ({ fetch }) => ({ me: await loadMe(fetch) });
