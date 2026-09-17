import type { GameRepository, StoredGameProgramOccurrence } from '../games/game-repository';
import type {
  StoredTrainingSeries,
  TrainingSeriesRepository,
} from '../training/training-series-repository';

export interface ProgramGameEvent extends StoredGameProgramOccurrence {
  type: 'game';
}

export interface ProgramTrainingEvent {
  durationMinutes: number;
  id: string;
  locationName: string;
  seriesId: string;
  startTime: string;
  type: 'training';
  date: string;
}

export type ProgramEvent = ProgramGameEvent | ProgramTrainingEvent;

const byDateAndTime = (left: ProgramEvent, right: ProgramEvent) =>
  `${left.date}T${left.startTime}`.localeCompare(`${right.date}T${right.startTime}`);

const trainingEvents = (series: StoredTrainingSeries): ProgramTrainingEvent[] =>
  series.occurrences.map((occurrence) => ({
    ...occurrence,
    id: `${series.id}:${occurrence.date}`,
    seriesId: series.id,
    type: 'training' as const,
  }));

export const readProgram = async (
  games: GameRepository,
  trainingSeries: TrainingSeriesRepository,
): Promise<ProgramEvent[]> => {
  const [gameOccurrences, series] = await Promise.all([
    games.findAllOccurrences(),
    trainingSeries.findAll(),
  ]);

  return [
    ...gameOccurrences.map((occurrence) => ({ ...occurrence, type: 'game' as const })),
    ...series.flatMap(trainingEvents),
  ].sort(byDateAndTime);
};
