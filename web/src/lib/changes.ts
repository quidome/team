// The list of changes (story 4): words, the summary line and the labels.
import type { Change, ChangeGame } from '#lib/api/client.js';
import { placesText, placesWarnings } from '#lib/duties.js';
import { kinds } from '#lib/reading.js';
import { formatDay, type Filter } from '#lib/schedule.js';

/** From this many new possible duty games, they show as one line that opens. New games of the
 * team always show one by one. */
export const summaryFrom = 5;

const kindWords = new Map<string, string>(kinds);

/** "Sat 31 Oct 16:00", or "Sat 31 Oct, time not known yet". */
export function when(game: ChangeGame): string {
	const day = formatDay(game.date);
	return game.start_time ? `${day} ${game.start_time}` : `${day}, time not known yet`;
}

function place(game: ChangeGame): string {
	return `${game.hall}, ${game.town}`;
}

/** The game the change is about: as now, or as last seen when it is no longer in the game list. */
export function gameOf(change: Change): ChangeGame | null {
	return change.after ?? change.before ?? null;
}

/** What happened, for example "moved" or "now a home game of another club team". */
export function kindLine(change: Change): string {
	const ofTeam = change.reason === 'team';
	if (change.kind === 'into_schedule')
		return ofTeam ? 'now a game of the team' : 'now a home game of another club team';
	if (change.kind === 'out_of_schedule')
		return ofTeam ? 'no longer a game of the team' : 'no longer a home game of another club team';
	return kindWords.get(change.kind) ?? change.kind;
}

/** The old and new values, one line each, for example "Sat 31 Oct 16:00 → Sun 1 Nov 13:30". */
export function details(change: Change): string[] {
	const { before, after } = change;
	if (change.kind === 'gone') return before ? [`was ${when(before)}`] : [];
	if (!before || !after) return after ? [`${when(after)} · ${place(after)}`] : [];
	const lines: string[] = [];
	const has = (difference: Change['differences'][number]) =>
		change.differences.includes(difference);
	if (has('date') || has('start_time')) lines.push(`${when(before)} → ${when(after)}`);
	else if (has('start_time_known')) lines.push(when(after));
	if (has('place')) lines.push(`${place(before)} → ${place(after)}`);
	if (has('home_and_away')) lines.push(`home team: ${before.home} → ${after.home}`);
	if (has('team')) lines.push(`was ${before.home} – ${before.away}`);
	if (lines.length === 0) lines.push(`${when(after)} · ${place(after)}`);
	return lines;
}

/** The changes to show one by one, and the new possible duty games for one summary line (empty
 * when there are fewer than `summaryFrom`). Both keep the order of the list. */
export function splitSummary(changes: Change[]): { single: Change[]; newDutyGames: Change[] } {
	const newDutyGames = changes.filter(
		(change) => change.kind === 'new' && change.reason === 'duty'
	);
	if (newDutyGames.length < summaryFrom) return { single: changes, newDutyGames: [] };
	return { single: changes.filter((change) => !newDutyGames.includes(change)), newDutyGames };
}

/** The label of the button that marks every change the filter shows as seen. */
export function seenAllLabel(filter: Filter, count: number): string {
	if (filter === 'all') return 'Mark all as seen';
	return count === 1 ? 'Mark this one as seen' : `Mark these ${count} as seen`;
}

/** The line for a list with no changes under the filter. */
export function noChangesLine(filter: Filter): string {
	if (filter === 'team') return 'No changes of the team not seen.';
	if (filter === 'duty') return 'No changes of possible duty games not seen.';
	return 'No changes not seen.';
}

/** "has duties · Table 1, 2: open, open" and the warnings, for a change of a game with places of
 * the team or a part it no longer needs; `null` for other changes. */
export function dutiesOf(change: Change): { line: string; warnings: string[] } | null {
	const places = change.places;
	if (!places) return null;
	const { schedule, drivers } = places;
	const has =
		(schedule && (schedule.of_team.length > 0 || !schedule.required)) ||
		(drivers && (drivers.places.length > 0 || !drivers.required));
	if (!has) return null;
	// A gone game says so in its line of what happened.
	const warnings = placesWarnings({ in_list: true, places });
	return { line: `has duties · ${placesText(places)}`, warnings };
}

/** Whether the page of the game exists: not for a game that left the schedule without places. */
export function hasPage(change: Change): boolean {
	return change.kind !== 'out_of_schedule' || !!change.places;
}
