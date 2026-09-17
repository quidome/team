import { eq } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import type { TrainingSeriesRepository } from '../../application/training/training-series-repository';
import { createDatabase } from './database';
import { createPostgresTrainingSeriesRepository } from './postgres-training-series-repository';
import { locations, trainingSeries } from './schema';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  describe.skip('PostgreSQL training series repository', () => {});
} else {
  describe('PostgreSQL training series repository', () => {
    const series = {
      durationMinutes: 90,
      endDate: '2096-09-01',
      locationName: 'Integration training court',
      startDate: '2096-08-18',
      startTime: '18:30',
      weekday: 2 as const,
    };
    const occurrences = [
      {
        date: '2096-08-18',
        durationMinutes: 90,
        locationName: series.locationName,
        startTime: '18:30',
      },
      {
        date: '2096-08-25',
        durationMinutes: 90,
        locationName: series.locationName,
        startTime: '18:30',
      },
      {
        date: '2096-09-01',
        durationMinutes: 90,
        locationName: series.locationName,
        startTime: '18:30',
      },
    ];
    const database = createDatabase(databaseUrl);
    const repository: TrainingSeriesRepository = createPostgresTrainingSeriesRepository(database);
    let seriesId: string | undefined;

    beforeAll(async () => {
      await database.delete(locations).where(eq(locations.name, series.locationName));
      await database.insert(locations).values({ name: series.locationName, travelMinutes: 20 });
    });

    afterAll(async () => {
      if (seriesId) {
        await database.delete(trainingSeries).where(eq(trainingSeries.id, seriesId));
      }
      await database.delete(locations).where(eq(locations.name, series.locationName));
      await database.close();
    });

    it('persists and retrieves a series with its generated occurrences', async () => {
      const stored = await repository.save(series, occurrences);
      seriesId = stored.id;

      await expect(repository.findById(stored.id)).resolves.toEqual(stored);
      await expect(repository.findAll()).resolves.toEqual([stored]);
    });
  });
}
