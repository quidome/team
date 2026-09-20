import type { TrainingOccurrence, TrainingSeries } from '../domain/training-series';
import type {
  StoredTrainingSeries,
  TrainingSeriesRepository,
} from '../application/training/training-series-repository';

export class InMemoryTrainingSeriesRepository implements TrainingSeriesRepository {
  private nextId = 1;
  private readonly series = new Map<string, StoredTrainingSeries>();
  private readonly standaloneOccurrences: TrainingOccurrence[] = [];

  async findAll(): Promise<StoredTrainingSeries[]> {
    return [...this.series.values()];
  }

  async findAllOccurrences(): Promise<TrainingOccurrence[]> {
    return [...this.standaloneOccurrences];
  }

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

  async saveOccurrence(occurrence: TrainingOccurrence): Promise<TrainingOccurrence> {
    const stored = { ...occurrence, id: `training-occurrence-${this.nextId++}` };

    this.standaloneOccurrences.push(stored);

    return stored;
  }
}
