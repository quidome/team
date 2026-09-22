export type AbsenceReason = 'illness' | 'injury' | 'other';
export type ParticipationOccurrenceType = 'game' | 'training';
export type ParticipationStatus = 'absent' | 'present';

export interface ParticipationRecord {
  absenceReason?: AbsenceReason;
  occurrenceId: string;
  occurrenceType: ParticipationOccurrenceType;
  playerId: string;
  status: ParticipationStatus;
}

export interface ParticipationRepository {
  findAll(): Promise<ParticipationRecord[]>;
  findByOccurrence(
    occurrenceType: ParticipationOccurrenceType,
    occurrenceId: string,
  ): Promise<ParticipationRecord[]>;
  saveMany(records: ParticipationRecord[]): Promise<ParticipationRecord[]>;
}
