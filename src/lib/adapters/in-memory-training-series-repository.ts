import type { TrainingOccurrence, TrainingSeries } from '../domain/training-series';
import type {
  StoredTrainingSeries,
  TrainingSeriesRepository,
} from '../application/training/training-series-repository';

export class InMemoryTrainingSeriesRepository implements TrainingSeriesRepository {
  private nextId = 1;
  private readonly series = new Map<string, StoredTrainingSeries>();

  async findById(id: string): Promise<StoredTrainingSeries | undefined> {
    return this.series.get(id);
  }

  async save(
    series: TrainingSeries,
    occurrences: TrainingOccurrence[],
  ): Promise<StoredTrainingSeries> {
    const stored = { id: `training-series-${this.nextId++}`, occurrences, series };

    this.series.set(stored.id, stored);

    return stored;
  }
}
