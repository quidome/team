import type { TrainingOccurrence, TrainingSeries } from '../../domain/training-series';

export interface StoredTrainingSeries {
  id: string;
  occurrences: TrainingOccurrence[];
  series: TrainingSeries;
}

export interface TrainingSeriesRepository {
  findAll(): Promise<StoredTrainingSeries[]>;
  findAllOccurrences(): Promise<TrainingOccurrence[]>;
  findById(id: string): Promise<StoredTrainingSeries | undefined>;
  findOccurrenceById(id: string): Promise<TrainingOccurrence | undefined>;
  save(series: TrainingSeries, occurrences: TrainingOccurrence[]): Promise<StoredTrainingSeries>;
  saveOccurrence(occurrence: TrainingOccurrence): Promise<TrainingOccurrence>;
  updateOccurrenceStatus(
    id: string,
    status: 'cancelled' | 'scheduled',
  ): Promise<TrainingOccurrence>;
}
