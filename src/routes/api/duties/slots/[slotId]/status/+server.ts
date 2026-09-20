import { json } from '@sveltejs/kit';

import { correctDutyStatus } from '$lib/application/duties/manage-duties';
import type { DutySlotStatus } from '$lib/application/duties/duty-repository';
import { currentDutyRepository } from '$lib/server/composition-root';

const statuses = new Set<DutySlotStatus>([
  'assigned',
  'cancelled',
  'completed',
  'incomplete',
  'open',
]);

export const POST = async ({ params, request }) => {
  try {
    const payload: unknown = await request.json();
    const status =
      typeof payload === 'object' && payload !== null
        ? (payload as Record<string, unknown>).status
        : undefined;

    if (typeof status !== 'string' || !statuses.has(status as DutySlotStatus)) {
      return json({ error: 'invalid_duty_status' }, { status: 400 });
    }

    return json(
      await correctDutyStatus(currentDutyRepository(), params.slotId, status as DutySlotStatus),
    );
  } catch (error) {
    if (error instanceof Error && error.message.includes('does not exist')) {
      return json({ error: 'duty_slot_not_found' }, { status: 400 });
    }

    throw error;
  }
};
