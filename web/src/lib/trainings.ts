// Words for trainings and the season totals (attendance at training, stories B and C; the games
// in the totals, story D of attendance at games).
import type { PlayerTotals, TrainingRow } from '#lib/api/client.js';
import { formatDay } from '#lib/schedule.js';

/** "Not opened", "All there", "2 absent" or "Cancelled"; on the card "Not opened yet". */
export function statusText(row: Pick<TrainingRow, 'status' | 'absent'>, card = false): string {
	switch (row.status) {
		case 'not_opened':
			return card ? 'Not opened yet' : 'Not opened';
		case 'all_there':
			return 'All there';
		case 'absent':
			return `${row.absent} absent`;
		case 'cancelled':
			return 'Cancelled';
		default:
			return row.status;
	}
}

/** " · Checked" after the status of a checked training, else "". */
export function checkedText(row: Pick<TrainingRow, 'checked'>): string {
	return row.checked ? ' · Checked' : '';
}

/** "3 training days not opened", "1 training day not opened". */
export function notOpenedText(count: number): string {
	return count === 1 ? '1 training day not opened' : `${count} training days not opened`;
}

/** "of 10 trainings", "of 1 training". */
export function ofTrainings(count: number): string {
	return count === 1 ? 'of 1 training' : `of ${count} trainings`;
}

/** "9 there · 1 absent, valid · 0 absent, not valid". */
export function countsText(
	totals: Pick<PlayerTotals, 'there' | 'absent_valid' | 'absent_not_valid'>
): string {
	return `${totals.there} there · ${totals.absent_valid} absent, valid · ${totals.absent_not_valid} absent, not valid`;
}

/** "of 9 games", "of 1 game". */
export function ofGames(count: number): string {
	return count === 1 ? 'of 1 game' : `of ${count} games`;
}

/** A missed training "Tue 6 Oct · Ill"; a missed game "Game Sat 10 Oct · Ill", or without a
 * reason "Game Sat 10 Oct · without a reason". */
export function missedText(missed: PlayerTotals['missed'][number]): string {
	if (missed.kind === 'training') return `${formatDay(missed.date)} · ${missed.reason ?? ''}`;
	return `Game ${formatDay(missed.date)} · ${missed.reason ?? 'without a reason'}`;
}

/** "Games count once they are checked. The start page asks for games from Sun 8 Nov." */
export function gamesFromText(date: string): string {
	return `Games count once they are checked. The start page asks for games from ${formatDay(date)}.`;
}
