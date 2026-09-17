import { generateTrainingOccurrences, type TrainingSeries } from '../../domain/training-series';
import type { StoredTrainingSeries, TrainingSeriesRepository } from './training-series-repository';

export const configureTrainingSeries = async (
  trainingSeries: TrainingSeriesRepository,
  series: TrainingSeries,
): Promise<StoredTrainingSeries> =>
  trainingSeries.save(series, generateTrainingOccurrences(series));
