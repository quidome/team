import type {
  ParticipationOccurrenceType,
  ParticipationRecord,
  ParticipationRepository,
} from '../application/participation/participation-repository';

const recordKey = (record: ParticipationRecord) =>
  [record.occurrenceType, record.occurrenceId, record.playerId].join('|');

export class InMemoryParticipationRepository implements ParticipationRepository {
  private readonly records = new Map<string, ParticipationRecord>();

  async findAll(): Promise<ParticipationRecord[]> {
    return [...this.records.values()];
  }

  async findByOccurrence(
    occurrenceType: ParticipationOccurrenceType,
    occurrenceId: string,
  ): Promise<ParticipationRecord[]> {
    return [...this.records.values()].filter(
      (record) => record.occurrenceType === occurrenceType && record.occurrenceId === occurrenceId,
    );
  }

  async saveMany(records: ParticipationRecord[]): Promise<ParticipationRecord[]> {
    for (const record of records) {
      this.records.set(recordKey(record), record);
    }

    return records;
  }
}
