import { json } from '@sveltejs/kit';

import {
  ClubConfigurationError,
  configureOwnClub,
  type OwnClubInput,
} from '$lib/application/clubs/configure-club';
import { currentAuditRepository, currentClubRepository } from '$lib/server/composition-root';

const readOwnClub = async (request: Request): Promise<OwnClubInput | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null) {
      const { name, sourceClubId } = payload as Record<string, unknown>;

      if (typeof name !== 'string' || !name.trim()) {
        return undefined;
      }

      if (sourceClubId === undefined || sourceClubId === null) {
        return { name: name.trim() };
      }

      if (typeof sourceClubId === 'number' && Number.isInteger(sourceClubId) && sourceClubId > 0) {
        return { name: name.trim(), sourceClubId };
      }
    }
  } catch {
    // The endpoint reports all malformed bodies as invalid club data.
  }

  return undefined;
};

export const GET = async () => {
  const clubs = currentClubRepository();
  const club = await clubs.findOwnClub();

  if (!club) {
    return json(null);
  }

  return json({ ...club, teamNames: await clubs.findTeamNames(club.name) });
};

export const PUT = async ({ request }) => {
  const input = await readOwnClub(request);

  if (!input) {
    return json({ error: 'invalid_club' }, { status: 400 });
  }

  try {
    const club = await configureOwnClub(currentClubRepository(), input);

    await currentAuditRepository().record({
      action: 'own_club_configured',
      entityId: club.name,
      entityType: 'club',
      metadata:
        club.sourceClubId === undefined
          ? { name: club.name }
          : { name: club.name, sourceClubId: club.sourceClubId },
    });

    return json(club);
  } catch (error) {
    if (error instanceof ClubConfigurationError) {
      return json({ error: error.message }, { status: 400 });
    }

    throw error;
  }
};
