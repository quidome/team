import { json } from '@sveltejs/kit';

import {
  recordAttendance,
  type RecordAttendanceCommand,
} from '$lib/application/participation/record-attendance';
import type {
  AbsenceReason,
  ParticipationOccurrenceType,
} from '$lib/application/participation/participation-repository';
import {
  currentAuditRepository,
  currentParticipationRepository,
} from '$lib/server/composition-root';

const absenceReasons = new Set<AbsenceReason>(['illness', 'injury', 'other']);
const occurrenceTypes = new Set<ParticipationOccurrenceType>(['game', 'training']);

const readCommand = async (request: Request): Promise<RecordAttendanceCommand | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload !== 'object' || payload === null) {
      return undefined;
    }

    const { absences, eligiblePlayerIds, occurrenceId, occurrenceType } = payload as Record<
      string,
      unknown
    >;

    if (
      !Array.isArray(absences) ||
      !absences.every(
        (absence) =>
          typeof absence === 'object' &&
          absence !== null &&
          typeof (absence as Record<string, unknown>).playerId === 'string' &&
          absenceReasons.has((absence as Record<string, unknown>).reason as AbsenceReason),
      ) ||
      !Array.isArray(eligiblePlayerIds) ||
      !eligiblePlayerIds.every((id) => typeof id === 'string') ||
      typeof occurrenceId !== 'string' ||
      !occurrenceId.trim() ||
      typeof occurrenceType !== 'string' ||
      !occurrenceTypes.has(occurrenceType as ParticipationOccurrenceType)
    ) {
      return undefined;
    }

    return {
      absences: absences.map((absence) => {
        const value = absence as Record<string, unknown>;

        return {
          playerId: (value.playerId as string).trim(),
          reason: value.reason as AbsenceReason,
        };
      }),
      eligiblePlayerIds: eligiblePlayerIds.map((id) => id.trim()),
      occurrenceId: occurrenceId.trim(),
      occurrenceType: occurrenceType as ParticipationOccurrenceType,
    };
  } catch {
    return undefined;
  }
};

export const GET = async ({ url }) => {
  const occurrenceId = url.searchParams.get('occurrenceId')?.trim();
  const occurrenceType = url.searchParams.get('occurrenceType');

  if (
    !occurrenceId ||
    !occurrenceType ||
    !occurrenceTypes.has(occurrenceType as ParticipationOccurrenceType)
  ) {
    return json({ error: 'invalid_participation_query' }, { status: 400 });
  }

  return json(
    await currentParticipationRepository().findByOccurrence(
      occurrenceType as ParticipationOccurrenceType,
      occurrenceId,
    ),
  );
};

export const POST = async ({ request }) => {
  const command = await readCommand(request);

  if (!command) {
    return json({ error: 'invalid_participation' }, { status: 400 });
  }

  try {
    const storedRecords = await recordAttendance(currentParticipationRepository(), command);

    await currentAuditRepository().record({
      action: 'attendance_recorded',
      entityId: `${command.occurrenceType}:${command.occurrenceId}`,
      entityType: 'participation',
      metadata: {
        absent: storedRecords.filter((record) => record.status === 'absent').length,
        occurrenceId: command.occurrenceId,
        occurrenceType: command.occurrenceType,
        present: storedRecords.filter((record) => record.status === 'present').length,
        records: storedRecords.length,
      },
    });

    return json(storedRecords);
  } catch (error) {
    if (error instanceof Error && error.message.includes('is not eligible')) {
      return json({ error: 'player_not_eligible' }, { status: 400 });
    }

    throw error;
  }
};
