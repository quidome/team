import type { GameRepository, StoredGameProgramOccurrence } from '../games/game-repository';
import type {
  StoredTrainingSeries,
  TrainingSeriesRepository,
} from '../training/training-series-repository';

export interface ProgramGameEvent extends StoredGameProgramOccurrence {
  type: 'game';
}

export interface ProgramTrainingEvent {
  date: string;
  durationMinutes: number;
  id: string;
  locationName: string;
  occurrenceId?: string;
  seriesId?: string;
  startTime: string;
  type: 'training';
}

export type ProgramEvent = ProgramGameEvent | ProgramTrainingEvent;

const byDateAndTime = (left: ProgramEvent, right: ProgramEvent) =>
  `${left.date}T${left.startTime}`.localeCompare(`${right.date}T${right.startTime}`);

const trainingEvent = (
  occurrence: StoredTrainingSeries['occurrences'][number],
  seriesId?: string,
): ProgramTrainingEvent => ({
  date: occurrence.date,
  durationMinutes: occurrence.durationMinutes,
  id: occurrence.id ?? `${seriesId ?? 'training'}:${occurrence.date}`,
  locationName: occurrence.locationName,
  ...(occurrence.id ? { occurrenceId: occurrence.id } : {}),
  ...(seriesId ? { seriesId } : {}),
  startTime: occurrence.startTime,
  type: 'training' as const,
});

const trainingEvents = (series: StoredTrainingSeries): ProgramTrainingEvent[] =>
  series.occurrences.map((occurrence) => trainingEvent(occurrence, series.id));

export const readProgram = async (
  games: GameRepository,
  trainingSeries: TrainingSeriesRepository,
): Promise<ProgramEvent[]> => {
  const [gameOccurrences, series, standaloneTrainingOccurrences] = await Promise.all([
    games.findAllOccurrences(),
    trainingSeries.findAll(),
    trainingSeries.findAllOccurrences(),
  ]);

  return [
    ...gameOccurrences.map((occurrence) => ({ ...occurrence, type: 'game' as const })),
    ...series.flatMap(trainingEvents),
    ...standaloneTrainingOccurrences.map((occurrence) => trainingEvent(occurrence)),
  ].sort(byDateAndTime);
};
