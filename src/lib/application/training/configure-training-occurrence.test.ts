import { describe, expect, it } from 'vitest';

import { InMemoryTrainingSeriesRepository } from '../../adapters/in-memory-training-series-repository';
import {
  cancelTrainingOccurrence,
  rescheduleTrainingOccurrence,
} from './configure-training-occurrence';

describe('training occurrence lifecycle', () => {
  it('cancels a scheduled occurrence', async () => {
    const training = new InMemoryTrainingSeriesRepository();
    const occurrence = await training.saveOccurrence({
      date: '2026-09-05',
      durationMinutes: 90,
      locationName: 'Home court',
      startTime: '18:00',
    });

    await expect(cancelTrainingOccurrence(training, occurrence.id ?? '')).resolves.toMatchObject({
      id: occurrence.id,
      status: 'cancelled',
    });
  });

  it('cancels the original occurrence when rescheduling', async () => {
    const training = new InMemoryTrainingSeriesRepository();
    const original = await training.saveOccurrence({
      date: '2026-09-05',
      durationMinutes: 90,
      locationName: 'Home court',
      startTime: '18:00',
    });

    const replacement = await rescheduleTrainingOccurrence(training, original.id ?? '', {
      date: '2026-09-12',
      durationMinutes: 75,
      locationName: 'Away court',
      startTime: '19:00',
    });

    expect(replacement).toMatchObject({
      date: '2026-09-12',
      durationMinutes: 75,
      status: 'scheduled',
    });
    await expect(training.findOccurrenceById(original.id ?? '')).resolves.toMatchObject({
      status: 'cancelled',
    });
  });
});
