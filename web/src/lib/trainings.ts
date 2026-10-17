// Words for trainings (attendance at training, story B).
import type { TrainingRow } from '#lib/api/client.js';

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
