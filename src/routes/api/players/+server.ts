import { json } from '@sveltejs/kit';

import { configurePlayer } from '$lib/application/players/configure-player';
import type { Player } from '$lib/application/players/player-repository';
import { currentAuditRepository, currentPlayerRepository } from '$lib/server/composition-root';

const isCalendarDate = (value: string) => {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    return false;
  }

  const date = new Date(`${value}T00:00:00.000Z`);

  return Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === value;
};

const readPlayer = async (request: Request): Promise<Player | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null) {
      const { associationId, birthDate, name } = payload as Record<string, unknown>;

      if (
        typeof associationId === 'string' &&
        associationId.trim() &&
        typeof birthDate === 'string' &&
        isCalendarDate(birthDate) &&
        typeof name === 'string' &&
        name.trim()
      ) {
        return {
          associationId: associationId.trim(),
          birthDate,
          name: name.trim(),
        };
      }
    }
  } catch {
    // The endpoint reports all malformed bodies as an invalid player.
  }

  return undefined;
};

export const POST = async ({ request }) => {
  const player = await readPlayer(request);

  if (!player) {
    return json({ error: 'invalid_player' }, { status: 400 });
  }

  const configuredPlayer = await configurePlayer(currentPlayerRepository(), player);

  await currentAuditRepository().record({
    action: 'player_configured',
    entityId: configuredPlayer.associationId,
    entityType: 'player',
    metadata: { birthDate: configuredPlayer.birthDate, name: configuredPlayer.name },
  });

  return json(configuredPlayer);
};
