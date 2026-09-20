import { json } from '@sveltejs/kit';

import { cancelGameOccurrence } from '$lib/application/games/configure-game';
import { currentAuditRepository, currentGameRepository } from '$lib/server/composition-root';

export const POST = async ({ params }) => {
  const cancelledOccurrence = await cancelGameOccurrence(
    currentGameRepository(),
    params.occurrenceId,
  );

  await currentAuditRepository().record({
    action: 'game_occurrence_cancelled',
    entityId: cancelledOccurrence.id,
    entityType: 'game_occurrence',
    metadata: { status: cancelledOccurrence.status },
  });

  return json(cancelledOccurrence);
};
