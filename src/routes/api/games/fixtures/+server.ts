import { json } from '@sveltejs/kit';

import { configureGameFixture } from '$lib/application/games/configure-game';
import type { GameFixture, SeasonHalf } from '$lib/application/games/game-repository';
import { currentAuditRepository, currentGameRepository } from '$lib/server/composition-root';

const readFixture = async (request: Request): Promise<GameFixture | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null) {
      const {
        awayTeamName,
        homeTeamName,
        isHome,
        opponentAddress,
        opponentName,
        opponentTravelMinutes,
        ourTeamName,
        seasonHalf,
        seasonStartingYear,
      } = payload as Record<string, unknown>;

      if (
        typeof awayTeamName === 'string' &&
        awayTeamName.trim() &&
        typeof homeTeamName === 'string' &&
        homeTeamName.trim() &&
        awayTeamName.trim() !== homeTeamName.trim()
      ) {
        const opponentFixture =
          typeof opponentName === 'string' &&
          opponentName.trim() &&
          typeof ourTeamName === 'string' &&
          ourTeamName.trim() &&
          typeof seasonStartingYear === 'number' &&
          Number.isInteger(seasonStartingYear) &&
          (seasonHalf === 'H1' || seasonHalf === 'H2') &&
          (isHome === undefined || typeof isHome === 'boolean') &&
          (opponentAddress === undefined || typeof opponentAddress === 'string') &&
          (opponentTravelMinutes === undefined ||
            (typeof opponentTravelMinutes === 'number' &&
              Number.isInteger(opponentTravelMinutes) &&
              opponentTravelMinutes >= 0));

        if (opponentName !== undefined && !opponentFixture) {
          return undefined;
        }

        return {
          awayTeamName: awayTeamName.trim(),
          homeTeamName: homeTeamName.trim(),
          ...(opponentFixture
            ? {
                ...(isHome === undefined ? {} : { isHome }),
                ...(opponentAddress === undefined
                  ? {}
                  : { opponentAddress: opponentAddress.trim() }),
                opponentName: opponentName.trim(),
                ...(opponentTravelMinutes === undefined ? {} : { opponentTravelMinutes }),
                ourTeamName: ourTeamName.trim(),
                seasonHalf: seasonHalf as SeasonHalf,
                seasonStartingYear,
              }
            : {}),
        };
      }
    }
  } catch {
    // The endpoint reports all malformed bodies as an invalid game fixture.
  }

  return undefined;
};

export const POST = async ({ request }) => {
  const fixture = await readFixture(request);

  if (!fixture) {
    return json({ error: 'invalid_game_fixture' }, { status: 400 });
  }

  const configuredFixture = await configureGameFixture(currentGameRepository(), fixture);

  await currentAuditRepository().record({
    action: 'game_fixture_configured',
    entityId: configuredFixture.id,
    entityType: 'game_fixture',
    metadata: {
      awayTeamName: configuredFixture.fixture.awayTeamName,
      homeTeamName: configuredFixture.fixture.homeTeamName,
    },
  });

  return json(configuredFixture);
};
