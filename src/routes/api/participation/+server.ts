import { json } from '@sveltejs/kit';

import {
  recordAttendance,
  type RecordAttendanceCommand,
} from '$lib/application/participation/record-attendance';
import type {
  AbsenceReason,
  ParticipationOccurrenceType,
} from '$lib/application/participation/participation-repository';
import { currentParticipationRepository } from '$lib/server/composition-root';

const absenceReasons = new Set<AbsenceReason>(['illness', 'injury', 'other']);
const occurrenceTypes = new Set<ParticipationOccurrenceType>(['game', 'training']);

const readCommand = async (request: Request): Promise<RecordAttendanceCommand | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload !== 'object' || payload === null) {
      return undefined;
    }

    const { absences, eligiblePlayerAssociationIds, occurrenceId, occurrenceType } =
      payload as Record<string, unknown>;

    if (
      !Array.isArray(absences) ||
      !absences.every(
        (absence) =>
          typeof absence === 'object' &&
          absence !== null &&
          typeof (absence as Record<string, unknown>).playerAssociationId === 'string' &&
          absenceReasons.has((absence as Record<string, unknown>).reason as AbsenceReason),
      ) ||
      !Array.isArray(eligiblePlayerAssociationIds) ||
      !eligiblePlayerAssociationIds.every((id) => typeof id === 'string') ||
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
          playerAssociationId: (value.playerAssociationId as string).trim(),
          reason: value.reason as AbsenceReason,
        };
      }),
      eligiblePlayerAssociationIds: eligiblePlayerAssociationIds.map((id) => id.trim()),
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
    return json(await recordAttendance(currentParticipationRepository(), command));
  } catch (error) {
    if (error instanceof Error && error.message.includes('is not eligible')) {
      return json({ error: 'player_not_eligible' }, { status: 400 });
    }

    throw error;
  }
};
