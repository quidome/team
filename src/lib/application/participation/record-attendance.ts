import type {
  AbsenceReason,
  ParticipationOccurrenceType,
  ParticipationRecord,
  ParticipationRepository,
} from './participation-repository';

export interface RecordAttendanceCommand {
  absences: Array<{ playerAssociationId: string; reason: AbsenceReason }>;
  eligiblePlayerAssociationIds: string[];
  occurrenceId: string;
  occurrenceType: ParticipationOccurrenceType;
}

export const recordAttendance = async (
  participation: ParticipationRepository,
  command: RecordAttendanceCommand,
): Promise<ParticipationRecord[]> => {
  const eligiblePlayers = new Set(command.eligiblePlayerAssociationIds);
  const absences = new Map(
    command.absences.map((absence) => [absence.playerAssociationId, absence.reason]),
  );

  for (const playerAssociationId of absences.keys()) {
    if (!eligiblePlayers.has(playerAssociationId)) {
      throw new Error(`${playerAssociationId} is not eligible for the occurrence`);
    }
  }

  const records = command.eligiblePlayerAssociationIds.map((playerAssociationId) => {
    const absenceReason = absences.get(playerAssociationId);

    return {
      ...(absenceReason === undefined ? {} : { absenceReason }),
      occurrenceId: command.occurrenceId,
      occurrenceType: command.occurrenceType,
      playerAssociationId,
      status: absenceReason === undefined ? ('present' as const) : ('absent' as const),
    };
  });

  return participation.saveMany(records);
};
