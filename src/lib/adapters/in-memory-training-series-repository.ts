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

  async findOccurrenceById(id: string): Promise<TrainingOccurrence | undefined> {
    for (const storedSeries of this.series.values()) {
      const occurrence = storedSeries.occurrences.find((candidate) => candidate.id === id);

      if (occurrence) {
        return occurrence;
      }
    }

    return this.standaloneOccurrences.find((occurrence) => occurrence.id === id);
  }

  async save(
    series: TrainingSeries,
    occurrences: TrainingOccurrence[],
  ): Promise<StoredTrainingSeries> {
    const id = `training-series-${this.nextId++}`;
    const storedOccurrences = occurrences.map((occurrence) => ({
      ...occurrence,
      id: `training-occurrence-${this.nextId++}`,
      seriesId: id,
      status: occurrence.status ?? 'scheduled',
    }));
    const stored = { id, occurrences: storedOccurrences, series };

    this.series.set(stored.id, stored);

    return stored;
  }

  async saveOccurrence(occurrence: TrainingOccurrence): Promise<TrainingOccurrence> {
    const stored = {
      ...occurrence,
      id: `training-occurrence-${this.nextId++}`,
      seriesId: undefined,
      status: occurrence.status ?? 'scheduled',
    };

    this.standaloneOccurrences.push(stored);

    return stored;
  }

  async updateOccurrenceStatus(
    id: string,
    status: 'cancelled' | 'scheduled',
  ): Promise<TrainingOccurrence> {
    const occurrence = await this.findOccurrenceById(id);

    if (!occurrence) {
      throw new Error(`Training occurrence ${id} does not exist`);
    }

    occurrence.status = status;

    return occurrence;
  }
}
