import { json } from '@sveltejs/kit';

import { cancelGameOccurrence } from '$lib/application/games/configure-game';
import { currentGameRepository } from '$lib/server/composition-root';

export const POST = async ({ params }) =>
  json(await cancelGameOccurrence(currentGameRepository(), params.occurrenceId));
