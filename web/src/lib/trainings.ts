// Words for trainings and the season totals (attendance at training, stories B and C).
import type { PlayerTotals, TrainingRow } from '#lib/api/client.js';

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
