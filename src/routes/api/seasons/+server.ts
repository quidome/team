import { json } from '@sveltejs/kit';

import { configureSeason } from '$lib/application/seasons/configure-season';
import { currentSeasonRepository } from '$lib/server/composition-root';

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

export const POST = async ({ request }) => {
  const startingYear = await readStartingYear(request);

  if (startingYear === undefined) {
    return json({ error: 'invalid_starting_year' }, { status: 400 });
  }

  const season = await configureSeason(currentSeasonRepository(), startingYear);

  return json(season);
};
