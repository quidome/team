// Duty places (stories A and B of duties): the words of the places of a game and of their players,
// for its page, its row on "Season", the start page and the list of changes.
import type { Choice, DutyKind, PlacesView, ScheduleGame, TeamView } from '#lib/api/client.js';

type SchedulePart = NonNullable<PlacesView['schedule']>;
type DriverPart = NonNullable<PlacesView['drivers']>;
type Place = SchedulePart['of_team'][number];

const kindNames: Record<Place['kind'], string> = {
	referee: 'Referee',
	table: 'Table',
	drive: 'Drivers'
};

/** Who did the duty of a filled place: "Anna", "Bram (was Anna)", "not done (was Anna)". */
export function playerText(player: NonNullable<Place['player']>): string {
	if (player.outcome === 'not_done') return `not done (was ${player.name})`;
	if (player.outcome === 'done_by' && player.done_by)
		return `${player.done_by.name} (was ${player.name})`;
	return player.name;
}

/** The player who does or did the duty of a filled place: the planned player, the player of
 * "Someone else", or `null` after "Nobody". */
export function doerOf(player: NonNullable<Place['player']>): number | null {
	if (player.outcome === 'not_done') return null;
	if (player.outcome === 'done_by' && player.done_by) return player.done_by.id;
	return player.id;
}

/** The player of each place, or "open". */
function fills(list: Place[]): string {
	return list.map((place) => (place.player ? playerText(place.player) : 'open')).join(', ');
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

/** The drivers of an away game of the team: "Drivers (4, proposed): Anna, 3 open", "all filled". */
function driversText(part: DriverPart): string {
	const filled = part.places.filter((place) => place.filled);
	if (part.places.length > 0 && filled.length === part.places.length) return 'all filled';
	const note = part.more_than_proposed ? ', more than proposed' : part.proposed ? ', proposed' : '';
	const names = filled.flatMap((place) => (place.player ? [playerText(place.player)] : []));
	const open = `${part.places.length - filled.length} open`;
	return `Drivers (${part.places.length}${note}): ${[...names, open].join(', ')}`;
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

/** The name of a place of the team: "Table 1", "Driver 2". */
export function placeName(kind: DutyKind, number: number): string {
	return `${kind === 'drive' ? 'Driver' : kindNames[kind]} ${number}`;
}

/** The totals of a player for the kind of a place: "2 done · 1 planned". */
export function totalText(choice: Choice, kind: DutyKind): string {
	const total = choice.totals[kind];
	return `${total.done} done · ${total.planned} planned`;
}

const clashKinds: Record<DutyKind, string> = {
	drive: 'Driver',
	table: 'Table',
	referee: 'Referee'
};

/** A clash of a player: "also Table 13:00", "plays 14:30", "plays (time not known)". */
export function clashText(clash: Choice['clashes'][number]): string {
	const what = clash.duty ? `also ${clashKinds[clash.duty]}` : 'plays';
	return clash.start ? `${what} ${clash.start}` : `${what} (time not known)`;
}

const notOfferedTexts = {
	on_this_place: 'on this place',
	at_game: 'already at this game',
	not_in_team: 'Not in the team on this date',
	clash: 'at another game at almost the same time'
} as const;

/** Why a player cannot be chosen for `place`, or `null` when the player can be chosen; the server
 * works it out with the checks of the saves (`offers`). `correcting`: "Someone else". */
export function notOffered(
	choice: Choice,
	place: Pick<Place, 'kind' | 'number'>,
	correcting: boolean
): string | null {
	const offer = choice.offers.find(
		(offer) => offer.kind === place.kind && offer.number === place.number
	);
	const reason = correcting ? offer?.correct : offer?.assign;
	return reason ? notOfferedTexts[reason] : null;
}
