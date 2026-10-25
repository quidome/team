// The start page (story B of start page and season): the words and the links of the open items.
import { resolve } from '$app/paths';
import type { OpenItemView } from '#lib/api/client.js';
import { formatDay } from '#lib/schedule.js';

/** "1 change not seen", "3 changes not seen". */
export function unseenLine(count: number): string {
	return count === 1 ? '1 change not seen' : `${count} changes not seen`;
}

/** The line of an open item, for example "Training Thu 22 Oct not checked". */
export function openItemText(item: OpenItemView): string {
	switch (item.kind) {
		case 'reading_held_back':
			return 'The last reading of the game list was held back.';
		case 'out_of_date':
			return `The games may be out of date since ${formatDay(item.since)}.`;
		case 'changes':
			return unseenLine(item.count);
		case 'not_opened':
			return item.count === 1
				? `Training day ${formatDay(item.first)} not opened`
				: `${item.count} training days not opened`;
		case 'not_checked':
			return item.count === 1
				? `Training ${formatDay(item.first)} not checked`
				: `${item.count} trainings not checked`;
	}
}

/** Where a tap on an open item goes: the "Game list" card, the list of changes, or the oldest
 * training day or training (with `?from=start`, so "Back" returns here). */
export function openItemLink(item: OpenItemView): string {
	switch (item.kind) {
		case 'reading_held_back':
		case 'out_of_date':
			return `${resolve('/season')}#game-list`;
		case 'changes':
			return resolve('/changes');
		case 'not_opened':
		case 'not_checked':
			return trainingLink(item.first);
	}
}

/** The training of `date`, opened from the start page: "Back" returns here. */
export function trainingLink(date: string): string {
	return `${resolve('/training/[date]', { date })}?from=start`;
}
