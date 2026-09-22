import { json } from '@sveltejs/kit';

import { configurePlayer } from '$lib/application/players/configure-player';
import type { Player } from '$lib/application/players/player-repository';
import { updatePlayer } from '$lib/application/players/update-player';
import { currentAuditRepository, currentPlayerRepository } from '$lib/server/composition-root';

const isCalendarDate = (value: string) => {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    return false;
  }

  const date = new Date(`${value}T00:00:00.000Z`);

  return Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === value;
};

const readPlayerFields = (payload: unknown): Omit<Player, 'id'> | undefined => {
  if (typeof payload !== 'object' || payload === null) {
    return undefined;
  }

  const { associationId, birthDate, firstName, lastName } = payload as Record<string, unknown>;

  if (
    typeof firstName === 'string' &&
    firstName.trim() &&
    (associationId === undefined || (typeof associationId === 'string' && associationId.trim())) &&
    (birthDate === undefined || (typeof birthDate === 'string' && isCalendarDate(birthDate))) &&
    (lastName === undefined || (typeof lastName === 'string' && lastName.trim()))
  ) {
    return {
      ...(typeof associationId === 'string' ? { associationId: associationId.trim() } : {}),
      ...(typeof birthDate === 'string' ? { birthDate } : {}),
      firstName: firstName.trim(),
      ...(typeof lastName === 'string' ? { lastName: lastName.trim() } : {}),
    };
  }

  return undefined;
};

const readPlayer = async (request: Request): Promise<Omit<Player, 'id'> | undefined> => {
  try {
    return readPlayerFields(await request.json());
  } catch {
    // The endpoint reports all malformed bodies as an invalid player.
    return undefined;
  }
};

const readPlayerUpdate = async (request: Request): Promise<Player | undefined> => {
  try {
    const payload: unknown = await request.json();
    const player = readPlayerFields(payload);

    if (!player || typeof payload !== 'object' || payload === null) {
      return undefined;
    }

    const { id } = payload as Record<string, unknown>;

    if (typeof id === 'string' && id.trim()) {
      return { ...player, id: id.trim() };
    }
  } catch {
    // The endpoint reports all malformed bodies as an invalid player.
  }

  return undefined;
};

export const DELETE = async ({ url }) => {
  const id = url.searchParams.get('id')?.trim();

  if (!id) {
    return json({ error: 'invalid_player' }, { status: 400 });
  }

  await currentPlayerRepository().deleteById(id);
  await currentAuditRepository().record({
    action: 'player_deleted',
    entityId: id,
    entityType: 'player',
    metadata: {},
  });

  return json({ deleted: true });
};

export const POST = async ({ request }) => {
  const player = await readPlayer(request);

  if (!player) {
    return json({ error: 'invalid_player' }, { status: 400 });
  }

  const configuredPlayer = await configurePlayer(currentPlayerRepository(), player);

  await currentAuditRepository().record({
    action: 'player_configured',
    entityId: configuredPlayer.id,
    entityType: 'player',
    metadata: {
      firstName: configuredPlayer.firstName,
      ...(configuredPlayer.birthDate ? { birthDate: configuredPlayer.birthDate } : {}),
    },
  });

  return json(configuredPlayer);
};

export const PUT = async ({ request }) => {
  const player = await readPlayerUpdate(request);

  if (!player) {
    return json({ error: 'invalid_player' }, { status: 400 });
  }

  try {
    const updatedPlayer = await updatePlayer(currentPlayerRepository(), player);

    await currentAuditRepository().record({
      action: 'player_updated',
      entityId: updatedPlayer.id,
      entityType: 'player',
      metadata: {
        firstName: updatedPlayer.firstName,
        ...(updatedPlayer.birthDate ? { birthDate: updatedPlayer.birthDate } : {}),
      },
    });

    return json(updatedPlayer);
  } catch {
    return json({ error: 'player_not_found' }, { status: 400 });
  }
};
