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

/** The message for an error answer of the API (`{ "error": "<code>" }`), or for the `error` of a
 * failed reading. */
export function messageFor(error: unknown, status: number): string {
	const code = (error as { error?: unknown } | undefined)?.error;
	if (code === 'source_unavailable') return 'The game list cannot be reached. Please try again.';
	if (code === 'source_format') return 'The game list has data the tool does not understand.';
	if (code === 'unknown_club') return 'This club is not in the game list.';
	if (code === 'not_an_own_team') return 'This team does not play for the club.';
	if (code === 'team_required') return 'Choose a team.';
	if (code === 'no_choice') return 'Choose your club first.';
	if (code === 'not_held_back')
		return 'This reading can no longer be confirmed. Read the game list again.';
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

export type Reading = components['schemas']['ReadingView'];
export type ReadingCounts = components['schemas']['CountsView'];
export type LatestReadings = components['schemas']['LatestReadings'];

export async function loadLatestReading(fetch: typeof globalThis.fetch): Promise<LatestReadings> {
	return dataOf(await api.GET('/api/readings/latest', { fetch }));
}

/** Reads the game list now. It waits when another reading is running. */
export async function readNow(): Promise<Reading> {
	return dataOf(await api.POST('/api/readings'));
}

/** "These games are really gone": reads again without holding back the held-back reading `id`. */
export async function confirmGone(id: number): Promise<Reading> {
	return dataOf(await api.POST('/api/readings/{id}/confirm-gone', { params: { path: { id } } }));
}

export type Schedule = components['schemas']['ScheduleView'];
export type ScheduleGame = components['schemas']['ScheduleGame'];

/** The schedule to show (story 3b), or null when there is none yet. */
export async function loadSchedule(fetch: typeof globalThis.fetch): Promise<Schedule | null> {
	return (await dataOf(await api.GET('/api/schedule', { fetch }))).schedule ?? null;
}

export type Change = components['schemas']['ChangeView'];
export type ChangeGame = components['schemas']['ChangeGame'];

/** The changes not seen yet (story 4). */
export async function loadChanges(fetch: typeof globalThis.fetch): Promise<Change[]> {
	return (await dataOf(await api.GET('/api/changes', { fetch }))).changes;
}

/** Marks these changes as seen; a change that a reading updated meanwhile stays. Gives the number
 * of changes still not seen. */
export async function markSeen(changes: Pick<Change, 'id' | 'version'>[]): Promise<number> {
	const seen = changes.map(({ id, version }) => ({ id, version }));
	return (await dataOf(await api.POST('/api/changes/seen', { body: { changes: seen } }))).open;
}
