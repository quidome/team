import type { TrainingOccurrence } from '../../domain/training-series';
import type { TrainingSeriesRepository } from './training-series-repository';

export const cancelTrainingOccurrence = async (
  training: TrainingSeriesRepository,
  occurrenceId: string,
): Promise<TrainingOccurrence> => {
  const occurrence = await training.findOccurrenceById(occurrenceId);

  if (!occurrence) {
    throw new Error(`Training occurrence ${occurrenceId} does not exist`);
  }

  return training.updateOccurrenceStatus(occurrenceId, 'cancelled');
};

export const rescheduleTrainingOccurrence = async (
  training: TrainingSeriesRepository,
  occurrenceId: string,
  occurrence: TrainingOccurrence,
): Promise<TrainingOccurrence> => {
  const existingOccurrence = await training.findOccurrenceById(occurrenceId);

  if (!existingOccurrence) {
    throw new Error(`Training occurrence ${occurrenceId} does not exist`);
  }

  const replacement = await training.saveOccurrence({
    date: occurrence.date,
    durationMinutes: occurrence.durationMinutes,
    locationName: occurrence.locationName,
    startTime: occurrence.startTime,
    status: 'scheduled',
  });

  await training.updateOccurrenceStatus(occurrenceId, 'cancelled');

  return replacement;
};
