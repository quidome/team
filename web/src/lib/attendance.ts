// The words of attendance at a game.

import type { CountView } from '#lib/api/client.js';

/** "6 coming · 1 not coming · 2 no answer"; from the start of the game (`started`) "6 there ·
 * 1 absent · 2 no answer". A part with 0 is left out. */
export function countText(count: CountView, started = false): string {
	return [
		[count.coming, started ? 'there' : 'coming'],
		[count.not_coming, started ? 'absent' : 'not coming'],
		[count.no_answer, 'no answer']
	]
		.filter(([number]) => number !== 0)
		.map(([number, words]) => `${number} ${words}`)
		.join(' · ');
}

/** The message after "Checked": how many players with no answer became absent; none for 0. */
export function settledText(settled: number): string | null {
	if (settled === 0) return null;
	return settled === 1
		? '1 player with no answer is absent.'
		: `${settled} players with no answer are absent.`;
}

/** The warning below "Not checked" of a game where nobody has an answer yet: "Mark as checked"
 * would make every player absent (the administration panel). `null` when a player has one. */
export function noAnswerWarning(count: CountView): string | null {
	if (count.coming !== 0 || count.not_coming !== 0 || count.no_answer === 0) return null;
	const players = count.no_answer === 1 ? 'the 1 player' : `all ${count.no_answer} players`;
	return `Nobody has an answer yet: marking as checked makes ${players} absent.`;
}
