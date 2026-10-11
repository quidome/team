// Typed client for the Rust API. Types come from `openapi.json` (`just api-types`).
import createClient from 'openapi-fetch';
import type { components, paths } from './schema.js';

export const api = createClient<paths>({ baseUrl: '', credentials: 'same-origin' });

export type Me = components['schemas']['Me'];

/** The logged-in user, or null when there is no valid session. */
export async function loadMe(fetch: typeof globalThis.fetch): Promise<Me | null> {
	const { data, response } = await api.GET('/api/me', { fetch });
	if (response.status === 401) return null;
	if (!data) throw new Error(`Unexpected answer from /api/me: ${response.status}`);
	return data;
}

export type Selection = components['schemas']['Selection'];
export type ClubView = components['schemas']['ClubView'];
export type TeamView = components['schemas']['TeamView'];
export type ClubTeams = components['schemas']['ClubTeams'];

/** A failed API call, with a message for the coordinator. */
export class ApiError extends Error {}

/** The message for an error answer of the API (`{ "error": "<code>" }`). */
function messageFor(error: unknown, status: number): string {
	const code = (error as { error?: unknown } | undefined)?.error;
	if (code === 'source_unavailable') return 'The game list cannot be reached. Please try again.';
	if (code === 'source_format') return 'The game list has data the tool does not understand.';
	if (code === 'unknown_club') return 'This club is not in the game list.';
	if (code === 'not_an_own_team') return 'This team does not play for the club.';
	if (code === 'team_required') return 'Choose a team.';
	if (status === 404) return 'This club is not in the game list.';
	return 'Something went wrong. Please try again.';
}

/** The answer's data, or an `ApiError` with a message for the coordinator. */
function dataOf<T>(result: { data?: T; error?: unknown; response: Response }): T {
	if (result.data === undefined)
		throw new ApiError(messageFor(result.error, result.response.status));
	return result.data;
}

export async function loadSelection(fetch: typeof globalThis.fetch): Promise<Selection> {
	return dataOf(await api.GET('/api/selection', { fetch }));
}

/** The selection for a page: `null` without login (the page then offers to log in). */
export async function loadPageSelection(event: {
	fetch: typeof globalThis.fetch;
	parent: () => Promise<{ me: Me | null }>;
}): Promise<{ selection: Selection | null }> {
	const { me } = await event.parent();
	return { selection: me ? await loadSelection(event.fetch) : null };
}

export async function searchClubs(search: string): Promise<ClubView[]> {
	return dataOf(await api.GET('/api/clubs', { params: { query: { search } } }));
}

export async function loadTeams(club: number): Promise<ClubTeams> {
	return dataOf(await api.GET('/api/clubs/{number}/teams', { params: { path: { number: club } } }));
}

export async function saveSelection(club: number, team: number | null): Promise<Selection> {
	return dataOf(await api.PUT('/api/selection', { body: { club, team } }));
}
