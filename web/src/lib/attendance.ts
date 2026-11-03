// The words of attendance at a game.

import type { CountView } from '#lib/api/client.js';

/** "6 coming · 1 not coming · 2 no answer"; a part with 0 is left out. */
export function countText(count: CountView): string {
	return [
		[count.coming, 'coming'],
		[count.not_coming, 'not coming'],
		[count.no_answer, 'no answer']
	]
		.filter(([number]) => number !== 0)
		.map(([number, words]) => `${number} ${words}`)
		.join(' · ');
}
