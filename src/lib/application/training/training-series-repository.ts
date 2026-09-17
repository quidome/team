import type { TrainingOccurrence, TrainingSeries } from '../../domain/training-series';

export interface StoredTrainingSeries {
  id: string;
  occurrences: TrainingOccurrence[];
  series: TrainingSeries;
}

export interface TrainingSeriesRepository {
  findById(id: string): Promise<StoredTrainingSeries | undefined>;
  save(series: TrainingSeries, occurrences: TrainingOccurrence[]): Promise<StoredTrainingSeries>;
}
