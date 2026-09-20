import { json } from '@sveltejs/kit';

import { cancelTrainingOccurrence } from '$lib/application/training/configure-training-occurrence';
import {
  currentAuditRepository,
  currentTrainingSeriesRepository,
} from '$lib/server/composition-root';

export const POST = async ({ params }) => {
  let cancelledOccurrence;

  try {
    cancelledOccurrence = await cancelTrainingOccurrence(
      currentTrainingSeriesRepository(),
      params.occurrenceId,
    );
  } catch (error) {
    if (error instanceof Error && error.message.includes('does not exist')) {
      return json({ error: 'training_occurrence_not_found' }, { status: 400 });
    }

    throw error;
  }

  await currentAuditRepository().record({
    action: 'training_occurrence_cancelled',
    entityId: cancelledOccurrence.id ?? 'unknown',
    entityType: 'training_occurrence',
    metadata: { status: cancelledOccurrence.status ?? 'unknown' },
  });

  return json(cancelledOccurrence);
};
