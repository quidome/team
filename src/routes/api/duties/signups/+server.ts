import { json } from '@sveltejs/kit';

import { recordDutySignup } from '$lib/application/duties/manage-duties';
import type {
  DutySignup,
  DutySignupStatus,
  DutyType,
} from '$lib/application/duties/duty-repository';
import { currentAuditRepository, currentDutyRepository } from '$lib/server/composition-root';

const dutyTypes = new Set<DutyType>(['driving', 'jury', 'referee']);
const signupStatuses = new Set<DutySignupStatus>(['selected', 'volunteer', 'waitlisted']);

const readSignup = async (request: Request): Promise<DutySignup | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload !== 'object' || payload === null) {
      return undefined;
    }

    const { dutyType, occurrenceId, playerAssociationId, status } = payload as Record<
      string,
      unknown
    >;

    if (
      typeof dutyType !== 'string' ||
      !dutyTypes.has(dutyType as DutyType) ||
      typeof occurrenceId !== 'string' ||
      !occurrenceId.trim() ||
      typeof playerAssociationId !== 'string' ||
      !playerAssociationId.trim() ||
      typeof status !== 'string' ||
      !signupStatuses.has(status as DutySignupStatus)
    ) {
      return undefined;
    }

    return {
      dutyType: dutyType as DutyType,
      occurrenceId: occurrenceId.trim(),
      playerAssociationId: playerAssociationId.trim(),
      status: status as DutySignupStatus,
    };
  } catch {
    return undefined;
  }
};

export const POST = async ({ request }) => {
  const signup = await readSignup(request);

  if (!signup) {
    return json({ error: 'invalid_duty_signup' }, { status: 400 });
  }

  try {
    const dutyView = await recordDutySignup(currentDutyRepository(), signup);

    await currentAuditRepository().record({
      action: 'duty_signup_recorded',
      entityId: `${signup.occurrenceId}:${signup.playerAssociationId}:${signup.dutyType}`,
      entityType: 'duty_signup',
      metadata: { status: signup.status },
    });

    return json(dutyView);
  } catch (error) {
    if (error instanceof Error && error.message.includes('does not exist')) {
      return json({ error: 'player_not_found' }, { status: 400 });
    }

    throw error;
  }
};
