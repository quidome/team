import type { TrainingOccurrence } from '$lib/domain/training-series';

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

export const readTrainingOccurrence = async (
  request: Request,
): Promise<TrainingOccurrence | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null) {
      const { date, durationMinutes, locationName, startTime } = payload as Record<string, unknown>;

      if (
        typeof date === 'string' &&
        isCalendarDate(date) &&
        typeof durationMinutes === 'number' &&
        Number.isInteger(durationMinutes) &&
        durationMinutes > 0 &&
        typeof locationName === 'string' &&
        locationName.trim() &&
        typeof startTime === 'string' &&
        isTime(startTime)
      ) {
        return {
          date,
          durationMinutes,
          locationName: locationName.trim(),
          startTime,
        };
      }
    }
  } catch {
    // The endpoint reports all malformed bodies as an invalid training occurrence.
  }

  return undefined;
};
