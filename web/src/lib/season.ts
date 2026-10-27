// The page "Season" (story A of start page and season): the labels, the parts, the empty lines
// and the week headings. The server sorts the events and gives each one its ISO week.
import type { EventView } from '#lib/api/client.js';

export type What = 'team' | 'duty' | 'training';
export type When = 'coming' | 'past';
export type Duties = 'entered' | 'not_entered' | 'open_places';
export type Labels = { what: What[]; when: When[]; duties: Duties[] };

export const whatLabels: { value: What; label: string }[] = [
	{ value: 'team', label: 'Games of the team' },
	{ value: 'duty', label: 'Possible duty games' },
	{ value: 'training', label: 'Trainings' }
];

export const whenLabels: { value: When; label: string }[] = [
	{ value: 'coming', label: 'Coming' },
	{ value: 'past', label: 'Past' }
];

export const dutyLabels: { value: Duties; label: string }[] = [
	{ value: 'entered', label: 'Entered' },
	{ value: 'not_entered', label: 'Not entered' },
	{ value: 'open_places', label: 'Open places' }
];

/** The labels at each visit: "Coming" on, no label of "What" or "Duties". */
export const startLabels = (): Labels => ({ what: [], when: ['coming'], duties: [] });

/** What "Season" keeps while a training or a game opened from it is shown: a return from it (with
 * "Back", the browser's back, or after "Delete training") shows the same labels and place. */
type Return = { season: string; key: string; labels: Labels; scrollY: number };
const returnKey = 'team.season.return';

/** The key of what a row opens: the date of a training, `game:<id>` for a game. */
export const trainingKey = (date: string) => date;
export const gameKey = (id: number | string) => `game:${id}`;

/** Keeps the labels and the place of the page before it opens the training or game of `key`. */
export function keepForReturn(season: string, key: string, labels: Labels, scrollY: number) {
	try {
		sessionStorage.setItem(returnKey, JSON.stringify({ season, key, labels, scrollY }));
	} catch {
		// No storage: the return starts with "Coming".
	}
}

/** What was kept for a return from the training or game of `key` in `season`, once; else
 * `null`. */
export function takeReturn(season: string, key: string | undefined): Return | null {
	try {
		const text = sessionStorage.getItem(returnKey);
		sessionStorage.removeItem(returnKey);
		const kept = text ? (JSON.parse(text) as Return) : null;
		return kept && kept.season === season && kept.key === key && kept.labels.duties ? kept : null;
	} catch {
		return null;
	}
}

/** What an event is, for the labels of "What". A game kept for its duties shows with the
 * possible duty games. */
export function whatOf(event: EventView): What {
	if (!event.game) return 'training';
	return event.game.reason === 'kept' ? 'duty' : event.game.reason;
}

/** Whether a game has the label `duty` of "Duties". "Entered" and "Not entered" are only for
 * games that need the schedule of the committee; "Not entered" is the rule of the open item. */
function hasDuty(event: EventView, duty: Duties): boolean {
	const places = event.game?.places;
	if (!places) return false;
	if (duty === 'entered') return !!places.schedule?.required && places.schedule.entered;
	if (duty === 'not_entered') return places.not_entered;
	return places.open_places > 0;
}

/** The events that show: their "What" is on (or no "What" is on), a label of "Duties" that is on
 * (or none is on: with one on, trainings and other games do not show), and a cancelled training
 * only while "Show cancelled" is on. */
export function visible(events: EventView[], labels: Labels, showCancelled: boolean): EventView[] {
	return events.filter(
		(event) =>
			(labels.what.length === 0 || labels.what.includes(whatOf(event))) &&
			(labels.duties.length === 0 || labels.duties.some((duty) => hasDuty(event, duty))) &&
			(showCancelled || event.training?.status !== 'cancelled')
	);
}

/** The parts to show: those of "When" that are on, or both when none is on. */
export function parts(labels: Labels): When[] {
	return labels.when.length === 0
		? ['coming', 'past']
		: whenLabels.map((l) => l.value).filter((when) => labels.when.includes(when));
}

/** The words for what the labels of "What" show. */
function noun(what: What[]): string {
	const team = what.includes('team');
	const duty = what.includes('duty');
	const training = what.includes('training');
	if (training && (team || duty)) return 'events';
	if (training) return 'trainings';
	if (team && duty) return 'games';
	if (team) return 'games of the team';
	if (duty) return 'possible duty games';
	return 'events';
}

/** "No coming trainings.", "No past possible duty games.", "No coming events.", "No coming games
 * with these labels." */
export function emptyLine(labels: Labels, part: When): string {
	// With a label of "Duties" on, only games show.
	if (labels.duties.length > 0) return `No ${part} games with these labels.`;
	return `No ${part} ${noun(labels.what)}.`;
}

/** "26 Oct" for a Dutch date such as "2026-10-26". */
function dayMonth(date: string): string {
	return new Intl.DateTimeFormat('en-GB', {
		day: 'numeric',
		month: 'short',
		timeZone: 'Europe/Amsterdam'
	}).format(new Date(`${date}T12:00:00Z`));
}

/** "Week 44 · 26 Oct – 1 Nov". */
export function weekHeading(
	event: Pick<EventView, 'week' | 'week_first_day' | 'week_last_day'>
): string {
	return `Week ${event.week} · ${dayMonth(event.week_first_day)} – ${dayMonth(event.week_last_day)}`;
}

/** The events grouped by date; the order of the server stays ("Next" on the start page). */
export function groupByDate(events: EventView[]): { date: string; events: EventView[] }[] {
	const days: { date: string; events: EventView[] }[] = [];
	for (const event of events) {
		const last = days.at(-1);
		if (last?.date === event.date) last.events.push(event);
		else days.push({ date: event.date, events: [event] });
	}
	return days;
}

/** The events grouped by week, then by date; the order of the server stays. */
export function groupByWeek(
	events: EventView[]
): { week: EventView; days: { date: string; events: EventView[] }[] }[] {
	const weeks: { week: EventView; events: EventView[] }[] = [];
	for (const event of events) {
		const week = weeks.at(-1);
		if (week?.week.week_first_day === event.week_first_day) week.events.push(event);
		else weeks.push({ week: event, events: [event] });
	}
	return weeks.map(({ week, events }) => ({ week, days: groupByDate(events) }));
}
