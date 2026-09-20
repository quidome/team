import { json } from '@sveltejs/kit';

import { configureSeason } from '$lib/application/seasons/configure-season';
import { currentAuditRepository, currentSeasonRepository } from '$lib/server/composition-root';

const readStartingYear = async (request: Request): Promise<number | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null && 'startingYear' in payload) {
      const { startingYear } = payload;

      if (typeof startingYear === 'number' && Number.isInteger(startingYear)) {
        return startingYear;
      }
    }
  } catch {
    // The endpoint reports all malformed bodies as an invalid starting year.
  }

  return undefined;
};

const readSeasonUpdate = async (
  request: Request,
): Promise<{ currentStartingYear: number; startingYear: number } | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null) {
      const { currentStartingYear, startingYear } = payload as Record<string, unknown>;

      if (
        typeof currentStartingYear === 'number' &&
        Number.isInteger(currentStartingYear) &&
        typeof startingYear === 'number' &&
        Number.isInteger(startingYear)
      ) {
        return { currentStartingYear, startingYear };
      }
    }
  } catch {
    // The endpoint reports malformed updates as invalid season data.
  }

  return undefined;
};

export const PUT = async ({ request }) => {
  const update = await readSeasonUpdate(request);

  if (!update) {
    return json({ error: 'invalid_season' }, { status: 400 });
  }

  const season = await currentSeasonRepository().updateStartingYear(
    update.currentStartingYear,
    update.startingYear,
  );
  await currentAuditRepository().record({
    action: 'season_updated',
    entityId: season.startingYear.toString(),
    entityType: 'season',
    metadata: { endingYear: season.endingYear },
  });

  return json(season);
};

export const DELETE = async ({ url }) => {
  const startingYear = Number(url.searchParams.get('startingYear'));

  if (!Number.isInteger(startingYear)) {
    return json({ error: 'invalid_starting_year' }, { status: 400 });
  }

  await currentSeasonRepository().deleteByStartingYear(startingYear);
  await currentAuditRepository().record({
    action: 'season_deleted',
    entityId: startingYear.toString(),
    entityType: 'season',
    metadata: {},
  });

  return json({ deleted: true });
};

export const POST = async ({ request }) => {
  const startingYear = await readStartingYear(request);

  if (startingYear === undefined) {
    return json({ error: 'invalid_starting_year' }, { status: 400 });
  }

  const season = await configureSeason(currentSeasonRepository(), startingYear);

  await currentAuditRepository().record({
    action: 'season_configured',
    entityId: season.startingYear.toString(),
    entityType: 'season',
    metadata: { endingYear: season.endingYear },
  });

  return json(season);
};
