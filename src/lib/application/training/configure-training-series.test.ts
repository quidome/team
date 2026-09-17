import { describe, expect, it } from 'vitest';

import { InMemoryTrainingSeriesRepository } from '../../adapters/in-memory-training-series-repository';
import { configureTrainingSeries } from './configure-training-series';

describe('configure training series', () => {
  it('stores a series and generated occurrences', async () => {
    const trainingSeries = new InMemoryTrainingSeriesRepository();

    const result = await configureTrainingSeries(trainingSeries, {
      durationMinutes: 90,
      endDate: '2026-09-01',
      locationName: 'Home court',
      startDate: '2026-08-18',
      startTime: '18:30',
      weekday: 2,
    });

    expect(result.series).toMatchObject({
      endDate: '2026-09-01',
      locationName: 'Home court',
      startDate: '2026-08-18',
    });
    expect(result.occurrences).toHaveLength(3);
  });
});
