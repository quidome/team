import { json } from '@sveltejs/kit';

import { rescheduleTrainingOccurrence } from '$lib/application/training/configure-training-occurrence';
import {
  currentAuditRepository,
  currentTrainingSeriesRepository,
} from '$lib/server/composition-root';
import { readTrainingOccurrence } from '$lib/server/training-occurrence-request';

export const POST = async ({ params, request }) => {
  const occurrence = await readTrainingOccurrence(request);

  if (!occurrence) {
    return json({ error: 'invalid_training_occurrence' }, { status: 400 });
  }

  let replacement;

  try {
    replacement = await rescheduleTrainingOccurrence(
      currentTrainingSeriesRepository(),
      params.occurrenceId,
      occurrence,
    );
  } catch (error) {
    if (error instanceof Error && error.message.includes('does not exist')) {
      return json({ error: 'training_occurrence_not_found' }, { status: 400 });
    }

    throw error;
  }

  await currentAuditRepository().record({
    action: 'training_occurrence_rescheduled',
    entityId: replacement.id ?? 'unknown',
    entityType: 'training_occurrence',
    metadata: {
      originalOccurrenceId: params.occurrenceId,
      replacementOccurrenceId: replacement.id ?? 'unknown',
    },
  });

  return json(replacement);
};
