import { json } from '@sveltejs/kit';

import { configureDuties, readDuties } from '$lib/application/duties/manage-duties';
import type { DutyRequirements } from '$lib/application/duties/duty-repository';
import {
  currentAuditRepository,
  currentDutyRepository,
  currentGameRepository,
} from '$lib/server/composition-root';

const readRequirements = async (
  request: Request,
): Promise<{ occurrenceId: string; requirements: DutyRequirements } | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload !== 'object' || payload === null) {
      return undefined;
    }

    const { drivingSlots, jurySlots, occurrenceId, refereeSlots } = payload as Record<
      string,
      unknown
    >;
    const slots = [drivingSlots, jurySlots, refereeSlots];

    if (
      typeof occurrenceId !== 'string' ||
      !occurrenceId.trim() ||
      !slots.every(
        (value) => typeof value === 'number' && Number.isInteger(value) && value >= 0 && value <= 2,
      )
    ) {
      return undefined;
    }

    return {
      occurrenceId: occurrenceId.trim(),
      requirements: {
        drivingSlots: drivingSlots as number,
        jurySlots: jurySlots as number,
        refereeSlots: refereeSlots as number,
      },
    };
  } catch {
    return undefined;
  }
};

export const GET = async ({ url }) => {
  const occurrenceId = url.searchParams.get('occurrenceId')?.trim();

  if (!occurrenceId) {
    return json({ error: 'invalid_duty_query' }, { status: 400 });
  }

  return json(await readDuties(currentDutyRepository(), currentGameRepository(), occurrenceId));
};

export const POST = async ({ request }) => {
  const input = await readRequirements(request);

  if (!input) {
    return json({ error: 'invalid_duty_requirements' }, { status: 400 });
  }

  try {
    const configuredDuties = await configureDuties(
      currentDutyRepository(),
      input.occurrenceId,
      input.requirements,
    );

    await currentAuditRepository().record({
      action: 'duty_requirements_configured',
      entityId: input.occurrenceId,
      entityType: 'game_occurrence',
      metadata: { ...input.requirements },
    });

    return json(configuredDuties);
  } catch (error) {
    if (error instanceof Error && error.message.includes('violates foreign key')) {
      return json({ error: 'game_occurrence_not_found' }, { status: 400 });
    }

    throw error;
  }
};
