// Duty places (story A of duties): the words of the places of a game, for its page, its row on
// "Season", the start page and the list of changes.
import type { PlacesView, ScheduleGame, TeamView } from '#lib/api/client.js';

type SchedulePart = NonNullable<PlacesView['schedule']>;
type DriverPart = NonNullable<PlacesView['drivers']>;
type Place = SchedulePart['of_team'][number];

const kindNames: Record<Place['kind'], string> = {
	referee: 'Referee',
	table: 'Table',
	drive: 'Drivers'
};

/** "open" for each place, or the player when it is filled (story B). */
function fills(list: Place[]): string {
	return list.map((place) => (place.filled ? 'filled' : 'open')).join(', ');
}

/** The places of the team at a home game of the club: "Referee 1: open · Table 1, 2: open, open",
 * "No places for the team", "all filled". */
function scheduleText(part: SchedulePart): string {
	if (part.of_team.length === 0) return 'No places for the team';
	if (part.of_team.every((place) => place.filled)) return 'all filled';
	const groups: string[] = [];
	for (const kind of ['referee', 'table'] as const) {
		const list = part.of_team.filter((place) => place.kind === kind);
		if (list.length === 0) continue;
		const numbers = list.map((place) => place.number).join(', ');
		groups.push(`${kindNames[kind]} ${numbers}: ${fills(list)}`);
	}
	return groups.join(' · ');
}

/** The drivers of an away game of the team: "Drivers (4, proposed): 4 open", "all filled". */
function driversText(part: DriverPart): string {
	const filled = part.places.filter((place) => place.filled).length;
	if (part.places.length > 0 && filled === part.places.length) return 'all filled';
	const note = part.more_than_proposed ? ', more than proposed' : part.proposed ? ', proposed' : '';
	return `Drivers (${part.places.length}${note}): ${part.places.length - filled} open`;
}

/** The places of the team in a line, for the rows of a game. */
export function placesText(view: PlacesView): string {
	const parts: string[] = [];
	if (view.schedule) parts.push(scheduleText(view.schedule));
	if (view.drivers) parts.push(driversText(view.drivers));
	return parts.join(' · ');
}

/** The places of the team on the page of a game: "M16 1: Referee 1, Table 1, Table 2", or "No
 * places for the team.". */
export function teamPlacesText(team: TeamView, part: SchedulePart): string {
	if (part.of_team.length === 0) return 'No places for the team.';
	const names = part.of_team.map((place) => `${kindNames[place.kind]} ${place.number}`);
	return `${team.name}: ${names.join(', ')}`;
}

/** The number of drivers on the page of a game: "Drivers: 4 (proposed)". */
export function driversNumber(part: DriverPart): string {
	const note = part.more_than_proposed
		? ' (more than proposed)'
		: part.proposed
			? ' (proposed)'
			: '';
	return `Drivers: ${part.places.length}${note}`;
}

/** What changed for the duties of a game since its places were chosen. */
export function placesWarnings(game: Pick<ScheduleGame, 'in_list' | 'places'>): string[] {
	const warnings: string[] = [];
	if (game.places?.schedule && !game.places.schedule.required)
		warnings.push('This game is no longer a home game of the club.');
	if (game.places?.drivers && !game.places.drivers.required)
		warnings.push('This game no longer needs drivers of the team.');
	if (!game.in_list) warnings.push('no longer in the game list');
	return warnings;
}

/** The places on the row of a game: the places of the team (`placesText`), then "· Schedule not
 * entered" while the schedule of a game that needs it is not entered; with no place chosen at all
 * only "Schedule not entered". */
export function rowPlacesText(view: PlacesView): string {
	const chosen = view.schedule?.places.some((place) => place.team) ?? false;
	if (view.not_entered && !chosen) {
		return view.drivers
			? `Schedule not entered · ${driversText(view.drivers)}`
			: 'Schedule not entered';
	}
	const text = placesText(view);
	return view.not_entered ? `${text} · Schedule not entered` : text;
}
