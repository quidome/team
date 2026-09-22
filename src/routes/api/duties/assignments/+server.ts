import { json } from '@sveltejs/kit';

import { assignDuty } from '$lib/application/duties/manage-duties';
import { currentAuditRepository, currentDutyRepository } from '$lib/server/composition-root';

const readAssignment = async (
  request: Request,
): Promise<{ playerId: string; slotId: string } | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload !== 'object' || payload === null) {
      return undefined;
    }

    const { playerId, slotId } = payload as Record<string, unknown>;

    if (
      typeof playerId !== 'string' ||
      !playerId.trim() ||
      typeof slotId !== 'string' ||
      !slotId.trim()
    ) {
      return undefined;
    }

    return { playerId: playerId.trim(), slotId: slotId.trim() };
  } catch {
    return undefined;
  }
};

export const POST = async ({ request }) => {
  const assignment = await readAssignment(request);

  if (!assignment) {
    return json({ error: 'invalid_duty_assignment' }, { status: 400 });
  }

  try {
    const assignedDuty = await assignDuty(
      currentDutyRepository(),
      assignment.slotId,
      assignment.playerId,
    );

    await currentAuditRepository().record({
      action: 'duty_assigned',
      entityId: assignment.slotId,
      entityType: 'duty_slot',
      metadata: { playerId: assignment.playerId },
    });

    return json(assignedDuty);
  } catch (error) {
    if (error instanceof Error && error.message.includes('does not exist')) {
      return json({ error: 'duty_assignment_target_not_found' }, { status: 400 });
    }

    if (error instanceof Error && error.message.includes('cannot be assigned')) {
      return json({ error: 'duty_slot_unavailable' }, { status: 400 });
    }

    throw error;
  }
};
