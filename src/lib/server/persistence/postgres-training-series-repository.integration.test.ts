import { eq } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import type { TrainingSeriesRepository } from '../../application/training/training-series-repository';
import { createDatabase } from './database';
import { createPostgresTrainingSeriesRepository } from './postgres-training-series-repository';
import { locations, trainingOccurrences, trainingSeries } from './schema';

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
    let standaloneOccurrenceId: string | undefined;

    beforeAll(async () => {
      const [existingLocation] = await database
        .select({ id: locations.id })
        .from(locations)
        .where(eq(locations.name, series.locationName))
        .limit(1);

      if (existingLocation) {
        await database
          .delete(trainingOccurrences)
          .where(eq(trainingOccurrences.locationId, existingLocation.id));
        await database
          .delete(trainingSeries)
          .where(eq(trainingSeries.locationId, existingLocation.id));
        await database.delete(locations).where(eq(locations.id, existingLocation.id));
      }

      await database.insert(locations).values({ name: series.locationName, travelMinutes: 20 });
    });

    afterAll(async () => {
      if (standaloneOccurrenceId) {
        await database
          .delete(trainingOccurrences)
          .where(eq(trainingOccurrences.id, standaloneOccurrenceId));
      }
      if (seriesId) {
        await database.delete(trainingSeries).where(eq(trainingSeries.id, seriesId));
      }
      await database.delete(locations).where(eq(locations.name, series.locationName));
      await database.close();
    });

    it('persists and retrieves a series with its generated occurrences', async () => {
      const stored = await repository.save(series, occurrences);
      seriesId = stored.id;

      const persisted = await repository.findById(stored.id);

      expect(persisted?.id).toBe(stored.id);
      expect(persisted?.series).toEqual(stored.series);
      const normalizedOccurrences = persisted?.occurrences.map(({ id, ...occurrence }) => {
        expect(id).toBeTruthy();
        return occurrence;
      });

      expect(normalizedOccurrences).toEqual(
        expect.arrayContaining(
          occurrences.map((occurrence) => ({ ...occurrence, status: 'scheduled' })),
        ),
      );
      expect(persisted?.occurrences).toHaveLength(occurrences.length);

      const firstOccurrenceId = persisted?.occurrences[0]?.id;
      expect(firstOccurrenceId).toBeTruthy();
      await expect(repository.findOccurrenceById(firstOccurrenceId ?? '')).resolves.toMatchObject({
        status: 'scheduled',
      });
      await expect(
        repository.updateOccurrenceStatus(firstOccurrenceId ?? '', 'cancelled'),
      ).resolves.toMatchObject({ status: 'cancelled' });

      const allSeries = await repository.findAll();

      expect(allSeries.some((candidate) => candidate.id === stored.id)).toBe(true);
    });

    it('persists and retrieves a standalone occurrence', async () => {
      const stored = await repository.saveOccurrence({
        date: '2096-08-20',
        durationMinutes: 60,
        locationName: series.locationName,
        startTime: '17:00',
      });
      standaloneOccurrenceId = stored.id;

      const standaloneOccurrences = await repository.findAllOccurrences();

      expect(standaloneOccurrences).toContainEqual(
        expect.objectContaining({
          date: '2096-08-20',
          durationMinutes: 60,
          id: stored.id,
          locationName: series.locationName,
          startTime: '17:00',
        }),
      );
    });
  });
}
