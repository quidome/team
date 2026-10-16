// Typed client for the Rust API. Types come from `openapi.json` (`just api-types`).
import createClient from 'openapi-fetch';
import type { components, paths } from './schema.js';
import { weekdays } from '#lib/players.js';

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
	const players: Record<string, string> = {
		first_name_required: 'A first name is needed.',
		until_before_from: 'The end must be after the start.',
		outside_season: 'This date is outside the season.',
		roles_overlap: 'This player already has a role on that date.',
		change_before_role_start:
			'The change must be after the start of the role. To change the role from its start, use "Correct" in the history.',
		not_in_team: 'This player is not in the team on that date.',
		end_before_start: 'The end time must be after the start time.',
		last_before_first: 'The last training date must be after the first.',
		weekly_trainings_overlap: 'There is already a weekly training on that day in these dates.',
		invalid_weekday: 'Choose a day of the week.',
		invalid_time: 'Enter a time such as 19:45.',
		reason_name_required: 'A name is needed.',
		reason_name_taken: 'There is already a reason with this name.'
	};
	if (typeof code === 'string' && code in players) return players[code];
	if (status === 404) return 'This no longer exists. Load the page again.';
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
	const result = await api.GET('/api/clubs/{number}/teams', { params: { path: { number: club } } });
	if (result.response.status === 404) throw new ApiError('This club is not in the game list.');
	return dataOf(result);
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

export type Players = components['schemas']['PlayersList'];
export type PlayerListItem = components['schemas']['PlayerListItem'];
export type PlayerDetail = components['schemas']['PlayerDetail'];
export type HistoryRole = components['schemas']['HistoryRole'];
export type RoleKind = components['schemas']['RoleKind'];
export type SavedPlayer = components['schemas']['SavedPlayer'];

/** All players of the club (attendance at training, story A). */
export async function loadPlayers(fetch: typeof globalThis.fetch): Promise<Players> {
	return dataOf(await api.GET('/api/players', { fetch }));
}

/** One player with its history, or null when it does not exist. */
export async function loadPlayer(
	fetch: typeof globalThis.fetch,
	id: number
): Promise<PlayerDetail | null> {
	const result = await api.GET('/api/players/{id}', { fetch, params: { path: { id } } });
	if (result.response.status === 404) return null;
	return dataOf(result);
}

export async function addPlayer(
	first_name: string,
	last_name: string | null,
	role: { role: RoleKind; from: string } | null
): Promise<SavedPlayer> {
	return dataOf(await api.POST('/api/players', { body: { first_name, last_name, role } }));
}

/** Saves the name and "Hidden". */
export async function changePlayer(
	id: number,
	first_name: string,
	last_name: string | null,
	hidden: boolean
): Promise<SavedPlayer> {
	return dataOf(
		await api.PUT('/api/players/{id}', {
			params: { path: { id } },
			body: { first_name, last_name, hidden }
		})
	);
}

export async function deletePlayer(id: number): Promise<void> {
	const result = await api.DELETE('/api/players/{id}', { params: { path: { id } } });
	if (!result.response.ok) throw new ApiError(messageFor(result.error, result.response.status));
}

/** "Add player" for a player from the list, and "Add to the team". */
export async function addRole(
	id: number,
	role: { role: RoleKind; from: string; until: string | null }
): Promise<TeamPage> {
	return dataOf(
		await api.POST('/api/players/{id}/roles', { params: { path: { id } }, body: role })
	);
}

/** "Correct" in the history: a role of the current season. */
export async function correctRole(
	id: number,
	roleId: number,
	role: { role: RoleKind; from: string; until: string | null }
): Promise<PlayerDetail> {
	return dataOf(
		await api.PUT('/api/players/{id}/roles/{role}', {
			params: { path: { id, role: roleId } },
			body: role
		})
	);
}

/** Brings these players of last season into this season, with the same role ("Add all"). */
export async function carryPlayers(players: number[]): Promise<TeamPage> {
	return dataOf(await api.POST('/api/players/carry', { body: { players } }));
}

export type TeamPage = components['schemas']['TeamPageView'];
export type Member = components['schemas']['MemberView'];
export type PlayerSummary = components['schemas']['PlayerSummary'];

/** The team of the current season (attendance at training, story A). */
export async function loadTeam(fetch: typeof globalThis.fetch): Promise<TeamPage> {
	return dataOf(await api.GET('/api/team', { fetch }));
}

/** "Change role": the role on `from` ends the day before, the new role starts on `from`. */
export async function changeRole(id: number, role: RoleKind, from: string): Promise<TeamPage> {
	return dataOf(
		await api.POST('/api/players/{id}/role-change', {
			params: { path: { id } },
			body: { role, from }
		})
	);
}

/** "Remove from the team": the role on `until` ends on that day. */
export async function leaveTeam(id: number, until: string): Promise<TeamPage> {
	return dataOf(
		await api.POST('/api/players/{id}/team-leave', { params: { path: { id } }, body: { until } })
	);
}

export type AbsenceReason = components['schemas']['AbsenceReason'];

export async function loadReasons(fetch: typeof globalThis.fetch): Promise<AbsenceReason[]> {
	return dataOf(await api.GET('/api/reasons', { fetch }));
}

export async function addReason(name: string, valid: boolean): Promise<AbsenceReason[]> {
	return dataOf(await api.POST('/api/reasons', { body: { name, valid } }));
}

export async function changeReason(
	reason: Pick<AbsenceReason, 'id' | 'name' | 'valid' | 'hidden'>
): Promise<AbsenceReason[]> {
	const { id, name, valid, hidden } = reason;
	return dataOf(
		await api.PUT('/api/reasons/{id}', { params: { path: { id } }, body: { name, valid, hidden } })
	);
}

export type WeeklyTrainings = components['schemas']['WeeklyTrainings'];
export type WeeklyTraining = components['schemas']['WeeklyTrainingView'];
export type WeeklyTrainingInput = components['schemas']['WeeklyTrainingInput'];

/** The weekly trainings of the current season (attendance at training, story A). */
export async function loadWeeklyTrainings(
	fetch: typeof globalThis.fetch
): Promise<WeeklyTrainings> {
	return dataOf(await api.GET('/api/weekly-trainings', { fetch }));
}

/** The proposed window of a weekly training on this day of the week (1 = Monday). */
export async function proposeWindow(weekday: number): Promise<{ from: string; until: string }> {
	return dataOf(
		await api.GET('/api/weekly-trainings/proposal', { params: { query: { weekday } } })
	);
}

/** The answer of a save, with the day of the week in the sentence of an overlap. */
function weeklyDataOf(
	result: { data?: WeeklyTrainings; error?: unknown; response: Response },
	weekday: number
): WeeklyTrainings {
	const code = (result.error as { error?: unknown } | undefined)?.error;
	if (code === 'weekly_trainings_overlap') {
		const day = weekdays.find((candidate) => candidate.value === weekday)?.label ?? 'that day';
		throw new ApiError(`There is already a weekly training on ${day} in these dates.`);
	}
	return dataOf(result);
}

export async function addWeeklyTraining(input: WeeklyTrainingInput): Promise<WeeklyTrainings> {
	return weeklyDataOf(await api.POST('/api/weekly-trainings', { body: input }), input.weekday);
}

export async function changeWeeklyTraining(
	id: number,
	input: WeeklyTrainingInput
): Promise<WeeklyTrainings> {
	return weeklyDataOf(
		await api.PUT('/api/weekly-trainings/{id}', { params: { path: { id } }, body: input }),
		input.weekday
	);
}

export async function deleteWeeklyTraining(id: number): Promise<WeeklyTrainings> {
	return dataOf(await api.DELETE('/api/weekly-trainings/{id}', { params: { path: { id } } }));
}
