import { json } from '@sveltejs/kit';

import { cancelGameOccurrence } from '$lib/application/games/configure-game';
import { currentAuditRepository, currentGameRepository } from '$lib/server/composition-root';

export const POST = async ({ params }) => {
  let cancelledOccurrence: Awaited<ReturnType<typeof cancelGameOccurrence>>;

  try {
    cancelledOccurrence = await cancelGameOccurrence(currentGameRepository(), params.occurrenceId);
  } catch (error) {
    if (error instanceof Error && error.message.includes('does not exist')) {
      return json({ error: 'game_occurrence_not_found' }, { status: 400 });
    }

    throw error;
  }

  await currentAuditRepository().record({
    action: 'game_occurrence_cancelled',
    entityId: cancelledOccurrence.id,
    entityType: 'game_occurrence',
    metadata: { status: cancelledOccurrence.status },
  });

  return json(cancelledOccurrence);
};
