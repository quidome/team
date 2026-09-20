import { json } from '@sveltejs/kit';

import {
  currentAuditRepository,
  currentTrainingSeriesRepository,
} from '$lib/server/composition-root';
import { readTrainingOccurrence } from '$lib/server/training-occurrence-request';

export const POST = async ({ request }) => {
  const occurrence = await readTrainingOccurrence(request);

  if (!occurrence) {
    return json({ error: 'invalid_training_occurrence' }, { status: 400 });
  }

  const storedOccurrence = await currentTrainingSeriesRepository().saveOccurrence(occurrence);

  await currentAuditRepository().record({
    action: 'training_occurrence_created',
    entityId: storedOccurrence.id ?? 'unknown',
    entityType: 'training_occurrence',
    metadata: {
      date: storedOccurrence.date,
      durationMinutes: storedOccurrence.durationMinutes,
      locationName: storedOccurrence.locationName,
      startTime: storedOccurrence.startTime,
    },
  });

  return json(storedOccurrence);
};
