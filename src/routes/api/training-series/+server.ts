import { json } from '@sveltejs/kit';

import { configureTrainingSeries } from '$lib/application/training/configure-training-series';
import type { TrainingSeries } from '$lib/domain/training-series';
import {
  currentAuditRepository,
  currentTrainingSeriesRepository,
} from '$lib/server/composition-root';

const isCalendarDate = (value: string) => {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    return false;
  }

  const date = new Date(`${value}T00:00:00.000Z`);

  return Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === value;
};

const isTime = (value: string) => {
  if (!/^\d{2}:\d{2}$/.test(value)) {
    return false;
  }

  const [hours, minutes] = value.split(':').map(Number);

  return hours >= 0 && hours < 24 && minutes >= 0 && minutes < 60;
};

const readTrainingSeries = async (request: Request): Promise<TrainingSeries | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null) {
      const { durationMinutes, endDate, locationName, startDate, startTime, weekday } =
        payload as Record<string, unknown>;

      if (
        typeof durationMinutes === 'number' &&
        Number.isInteger(durationMinutes) &&
        durationMinutes > 0 &&
        typeof endDate === 'string' &&
        isCalendarDate(endDate) &&
        typeof locationName === 'string' &&
        locationName.trim() &&
        typeof startDate === 'string' &&
        isCalendarDate(startDate) &&
        typeof startTime === 'string' &&
        isTime(startTime) &&
        typeof weekday === 'number' &&
        Number.isInteger(weekday) &&
        weekday >= 1 &&
        weekday <= 7
      ) {
        return {
          durationMinutes,
          endDate,
          locationName: locationName.trim(),
          startDate,
          startTime,
          weekday: weekday as TrainingSeries['weekday'],
        };
      }
    }
  } catch {
    // The endpoint reports all malformed bodies as an invalid training series.
  }

  return undefined;
};

export const POST = async ({ request }) => {
  const series = await readTrainingSeries(request);

  if (!series) {
    return json({ error: 'invalid_training_series' }, { status: 400 });
  }

  const configuredSeries = await configureTrainingSeries(currentTrainingSeriesRepository(), series);

  await currentAuditRepository().record({
    action: 'training_series_configured',
    entityId: configuredSeries.id,
    entityType: 'training_series',
    metadata: {
      endDate: configuredSeries.series.endDate,
      locationName: configuredSeries.series.locationName,
      startDate: configuredSeries.series.startDate,
      occurrences: configuredSeries.occurrences.length,
    },
  });

  return json(configuredSeries);
};
