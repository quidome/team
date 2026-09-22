import type {
  AbsenceReason,
  ParticipationOccurrenceType,
  ParticipationRecord,
  ParticipationRepository,
} from './participation-repository';

export interface RecordAttendanceCommand {
  absences: Array<{ playerId: string; reason: AbsenceReason }>;
  eligiblePlayerIds: string[];
  occurrenceId: string;
  occurrenceType: ParticipationOccurrenceType;
}

export const recordAttendance = async (
  participation: ParticipationRepository,
  command: RecordAttendanceCommand,
): Promise<ParticipationRecord[]> => {
  const eligiblePlayers = new Set(command.eligiblePlayerIds);
  const absences = new Map(command.absences.map((absence) => [absence.playerId, absence.reason]));

  for (const playerId of absences.keys()) {
    if (!eligiblePlayers.has(playerId)) {
      throw new Error(`${playerId} is not eligible for the occurrence`);
    }
  }

  const records = command.eligiblePlayerIds.map((playerId) => {
    const absenceReason = absences.get(playerId);

    return {
      ...(absenceReason === undefined ? {} : { absenceReason }),
      occurrenceId: command.occurrenceId,
      occurrenceType: command.occurrenceType,
      playerId,
      status: absenceReason === undefined ? ('present' as const) : ('absent' as const),
    };
  });

  return participation.saveMany(records);
};
