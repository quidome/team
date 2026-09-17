import { eq } from 'drizzle-orm';

import type { TrainingOccurrence, TrainingSeries, Weekday } from '../../domain/training-series';
import type {
  StoredTrainingSeries,
  TrainingSeriesRepository,
} from '../../application/training/training-series-repository';
import { createDatabase } from './database';
import { locations, trainingOccurrences, trainingSeries } from './schema';

type Database = ReturnType<typeof createDatabase>;

const readWeekday = (value: number): Weekday => {
  if (value < 1 || value > 7 || !Number.isInteger(value)) {
    throw new Error(`Invalid stored weekday: ${value}`);
  }

  return value as Weekday;
};

export const createPostgresTrainingSeriesRepository = (
  database: Database,
): TrainingSeriesRepository => ({
  async findAll(): Promise<StoredTrainingSeries[]> {
    const seriesRows = await database.select({ id: trainingSeries.id }).from(trainingSeries);

    return (await Promise.all(seriesRows.map((series) => this.findById(series.id)))).filter(
      (series): series is StoredTrainingSeries => series !== undefined,
    );
  },

  async findById(id: string): Promise<StoredTrainingSeries | undefined> {
    const [storedSeries] = await database
      .select({
        durationMinutes: trainingSeries.durationMinutes,
        endDate: trainingSeries.endDate,
        locationName: locations.name,
        startDate: trainingSeries.startDate,
        startTime: trainingSeries.startTime,
        weekday: trainingSeries.weekday,
      })
      .from(trainingSeries)
      .innerJoin(locations, eq(trainingSeries.locationId, locations.id))
      .where(eq(trainingSeries.id, id))
      .limit(1);

    if (!storedSeries) {
      return undefined;
    }

    const storedOccurrences = await database
      .select({
        date: trainingOccurrences.date,
        durationMinutes: trainingOccurrences.durationMinutes,
        locationName: locations.name,
        startTime: trainingOccurrences.startTime,
      })
      .from(trainingOccurrences)
      .innerJoin(locations, eq(trainingOccurrences.locationId, locations.id))
      .where(eq(trainingOccurrences.seriesId, id));

    return {
      id,
      occurrences: storedOccurrences,
      series: {
        durationMinutes: storedSeries.durationMinutes,
        endDate: storedSeries.endDate,
        locationName: storedSeries.locationName,
        startDate: storedSeries.startDate,
        startTime: storedSeries.startTime,
        weekday: readWeekday(storedSeries.weekday),
      },
    };
  },

  async save(
    series: TrainingSeries,
    occurrences: TrainingOccurrence[],
  ): Promise<StoredTrainingSeries> {
    const [location] = await database
      .select({ id: locations.id })
      .from(locations)
      .where(eq(locations.name, series.locationName))
      .limit(1);

    if (!location) {
      throw new Error(`Location ${series.locationName} does not exist`);
    }

    const [storedSeries] = await database
      .insert(trainingSeries)
      .values({
        durationMinutes: series.durationMinutes,
        endDate: series.endDate,
        locationId: location.id,
        startDate: series.startDate,
        startTime: series.startTime,
        weekday: series.weekday,
      })
      .returning({ id: trainingSeries.id });

    if (!storedSeries) {
      throw new Error('PostgreSQL did not return the stored training series');
    }

    if (occurrences.length > 0) {
      await database.insert(trainingOccurrences).values(
        occurrences.map((occurrence) => ({
          date: occurrence.date,
          durationMinutes: occurrence.durationMinutes,
          locationId: location.id,
          seriesId: storedSeries.id,
          startTime: occurrence.startTime,
        })),
      );
    }

    return { id: storedSeries.id, occurrences, series };
  },
});
