import { json } from '@sveltejs/kit';

import { scheduleGameOccurrence } from '$lib/application/games/configure-game';
import type { GameOccurrence } from '$lib/application/games/game-repository';
import { currentAuditRepository, currentGameRepository } from '$lib/server/composition-root';

const isCalendarDate = (value: string) => {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    return false;
  }

  const date = new Date(`${value}T00:00:00.000Z`);

  return Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === value;
};

const isTime = (value: string) => {
  if (!/^\d{2}:\d{2}$/.test(value)) {
    return false;
  }

  const [hours, minutes] = value.split(':').map(Number);

  return hours >= 0 && hours < 24 && minutes >= 0 && minutes < 60;
};

const readOccurrence = async (
  request: Request,
): Promise<{ fixtureId: string; occurrence: GameOccurrence } | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null) {
      const { arrivalBufferMinutes, date, fixtureId, locationName, startTime, travelMinutes } =
        payload as Record<string, unknown>;

      if (
        typeof arrivalBufferMinutes === 'number' &&
        Number.isInteger(arrivalBufferMinutes) &&
        arrivalBufferMinutes >= 0 &&
        typeof date === 'string' &&
        isCalendarDate(date) &&
        typeof fixtureId === 'string' &&
        fixtureId.trim() &&
        typeof locationName === 'string' &&
        locationName.trim() &&
        typeof startTime === 'string' &&
        isTime(startTime) &&
        typeof travelMinutes === 'number' &&
        Number.isInteger(travelMinutes) &&
        travelMinutes >= 0
      ) {
        return {
          fixtureId: fixtureId.trim(),
          occurrence: {
            arrivalBufferMinutes,
            date,
            locationName: locationName.trim(),
            startTime,
            travelMinutes,
          },
        };
      }
    }
  } catch {
    // The endpoint reports all malformed bodies as an invalid game occurrence.
  }

  return undefined;
};

export const POST = async ({ request }) => {
  const input = await readOccurrence(request);

  if (!input) {
    return json({ error: 'invalid_game_occurrence' }, { status: 400 });
  }

  const storedOccurrence = await scheduleGameOccurrence(
    currentGameRepository(),
    input.fixtureId,
    input.occurrence,
  );

  await currentAuditRepository().record({
    action: 'game_occurrence_created',
    entityId: storedOccurrence.id,
    entityType: 'game_occurrence',
    metadata: {
      date: storedOccurrence.date,
      fixtureId: storedOccurrence.fixtureId,
      locationName: storedOccurrence.locationName,
    },
  });

  return json(storedOccurrence);
};
