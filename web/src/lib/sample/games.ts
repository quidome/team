// Example games for the sample page (step 3 of the base). Not real data and not stored;
// story 3 of the epic replaces the sample page with the real schedule.

export interface SampleGame {
	id: number;
	/** ISO date, Dutch local time. */
	date: string;
	/** "HH:MM", or null when the game list has no start time yet. */
	time: string | null;
	home: string;
	away: string;
	hall: string;
	town: string;
	/** A game of the team (home or away); otherwise a home game of another club team. */
	ofTeam: boolean;
	/** No longer in the game list (shown crossed out until the change is seen). */
	gone?: boolean;
}

export const sampleGames: SampleGame[] = [
	{
		id: 1,
		date: '2026-10-17',
		time: '10:00',
		home: 'Blue Drakes X12 1',
		away: 'Quick Runners X12 1',
		hall: 'Bulwijk',
		town: 'Woerden',
		ofTeam: false
	},
	{
		id: 2,
		date: '2026-10-17',
		time: '12:15',
		home: 'Quick Runners M16 1',
		away: 'Blue Drakes M16 1',
		hall: 'Sporthal Wolfskamer',
		town: 'Huizen',
		ofTeam: true
	},
	{
		id: 3,
		date: '2026-10-17',
		time: '14:00',
		home: 'Blue Drakes M18 1',
		away: 'Green Giants M18 1',
		hall: 'Bulwijk',
		town: 'Woerden',
		ofTeam: false
	},
	{
		id: 4,
		date: '2026-10-24',
		time: '10:00',
		home: 'Blue Drakes M16 1',
		away: 'Green Giants M16 1',
		hall: 'Bulwijk',
		town: 'Woerden',
		ofTeam: true
	},
	{
		id: 5,
		date: '2026-10-24',
		time: null,
		home: 'Blue Drakes X14 1',
		away: 'Hoops X14 1',
		hall: 'Bulwijk',
		town: 'Woerden',
		ofTeam: false
	},
	{
		id: 6,
		date: '2026-10-31',
		time: '14:00',
		home: 'Blue Drakes M18 1',
		away: 'Hoops M18 1',
		hall: 'Bulwijk',
		town: 'Woerden',
		ofTeam: false,
		gone: true
	},
	{
		id: 7,
		date: '2026-10-31',
		time: '16:30',
		home: 'Blue Drakes MSE 1',
		away: 'Green Giants MSE 1',
		hall: 'Bulwijk',
		town: 'Woerden',
		ofTeam: false
	},
	{
		id: 8,
		date: '2026-11-07',
		time: '13:30',
		home: 'Hoops M16 1',
		away: 'Blue Drakes M16 1',
		hall: 'De Basis',
		town: 'Utrecht',
		ofTeam: true
	}
];

export type Filter = 'all' | 'team' | 'duty';

export function filterGames(games: SampleGame[], filter: Filter): SampleGame[] {
	if (filter === 'team') return games.filter((game) => game.ofTeam);
	if (filter === 'duty') return games.filter((game) => !game.ofTeam);
	return games;
}

/** Groups games by date, in date and time order. Games without a time come last in their day. */
export function groupByDate(games: SampleGame[]): { date: string; games: SampleGame[] }[] {
	const sorted = [...games].sort(
		(a, b) => a.date.localeCompare(b.date) || (a.time ?? '99').localeCompare(b.time ?? '99')
	);
	const groups: { date: string; games: SampleGame[] }[] = [];
	for (const game of sorted) {
		const last = groups.at(-1);
		if (last?.date === game.date) last.games.push(game);
		else groups.push({ date: game.date, games: [game] });
	}
	return groups;
}

/** "Sat 17 Oct" */
export function formatDay(date: string): string {
	return new Intl.DateTimeFormat('en-GB', {
		weekday: 'short',
		day: 'numeric',
		month: 'short',
		timeZone: 'Europe/Amsterdam'
	}).format(new Date(`${date}T12:00:00Z`));
}
