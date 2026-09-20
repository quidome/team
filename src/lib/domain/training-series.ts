export type Weekday = 1 | 2 | 3 | 4 | 5 | 6 | 7;
export type TrainingOccurrenceStatus = 'cancelled' | 'scheduled';

export interface TrainingSeries {
  durationMinutes: number;
  endDate: string;
  locationName: string;
  startDate: string;
  startTime: string;
  weekday: Weekday;
}

export interface TrainingOccurrence {
  date: string;
  id?: string;
  seriesId?: string;
  durationMinutes: number;
  locationName: string;
  startTime: string;
  status?: TrainingOccurrenceStatus;
}

const parseDate = (value: string) => {
  const date = new Date(`${value}T00:00:00.000Z`);

  if (!Number.isFinite(date.getTime()) || date.toISOString().slice(0, 10) !== value) {
    throw new Error(`Invalid calendar date: ${value}`);
  }

  return date;
};

const formatDate = (date: Date) => date.toISOString().slice(0, 10);

const isoWeekday = (date: Date): Weekday => {
  const day = date.getUTCDay();

  return (day === 0 ? 7 : day) as Weekday;
};

export const generateTrainingOccurrences = (series: TrainingSeries): TrainingOccurrence[] => {
  const currentDate = parseDate(series.startDate);
  const endDate = parseDate(series.endDate);
  const occurrences: TrainingOccurrence[] = [];

  while (currentDate <= endDate) {
    if (isoWeekday(currentDate) === series.weekday) {
      occurrences.push({
        date: formatDate(currentDate),
        durationMinutes: series.durationMinutes,
        locationName: series.locationName,
        startTime: series.startTime,
      });
    }

    currentDate.setUTCDate(currentDate.getUTCDate() + 1);
  }

  return occurrences;
};
