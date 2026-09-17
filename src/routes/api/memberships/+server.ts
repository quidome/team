import { json } from '@sveltejs/kit';

import { configureMembership } from '$lib/application/memberships/configure-membership';
import type { Membership } from '$lib/application/memberships/membership-repository';
import { currentMembershipRepository } from '$lib/server/composition-root';

const readMembership = async (request: Request): Promise<Membership | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null) {
      const {
        jerseyNumber,
        participationType,
        playerAssociationId,
        relationship,
        seasonStartingYear,
        status,
        teamName,
      } = payload as Record<string, unknown>;

      if (
        (jerseyNumber === undefined ||
          (typeof jerseyNumber === 'number' &&
            Number.isInteger(jerseyNumber) &&
            jerseyNumber > 0)) &&
        (participationType === 'trains_and_plays' || participationType === 'trains_only') &&
        typeof playerAssociationId === 'string' &&
        playerAssociationId.trim() &&
        (relationship === 'primary' || relationship === 'secondary') &&
        typeof seasonStartingYear === 'number' &&
        Number.isInteger(seasonStartingYear) &&
        (status === 'active' || status === 'inactive') &&
        typeof teamName === 'string' &&
        teamName.trim()
      ) {
        return {
          ...(jerseyNumber === undefined ? {} : { jerseyNumber }),
          participationType,
          playerAssociationId: playerAssociationId.trim(),
          relationship,
          seasonStartingYear,
          status,
          teamName: teamName.trim(),
        };
      }
    }
  } catch {
    // The endpoint reports all malformed bodies as an invalid membership.
  }

  return undefined;
};

export const POST = async ({ request }) => {
  const membership = await readMembership(request);

  if (!membership) {
    return json({ error: 'invalid_membership' }, { status: 400 });
  }

  return json(await configureMembership(currentMembershipRepository(), membership));
};
