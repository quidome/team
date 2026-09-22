import type {
  GameImportProvenance,
  GameImportRepository,
} from '../application/imports/game-import-repository';

export class InMemoryGameImportRepository implements GameImportRepository {
  private nextId = 1;
  readonly records: (GameImportProvenance & { id: string })[] = [];

  async save(provenance: GameImportProvenance) {
    const existing = this.records.find((record) => record.occurrenceId === provenance.occurrenceId);

    if (existing) {
      Object.assign(existing, provenance);

      return existing;
    }

    const stored = { ...provenance, id: `game-import-${this.nextId++}` };

    this.records.push(stored);

    return stored;
  }
}
