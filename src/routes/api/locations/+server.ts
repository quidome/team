import { json } from '@sveltejs/kit';

import { configureLocation } from '$lib/application/locations/configure-location';
import type { Location } from '$lib/application/locations/location-repository';
import { currentLocationRepository } from '$lib/server/composition-root';

const readLocation = async (request: Request): Promise<Location | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null) {
      const { name, travelMinutes } = payload as Record<string, unknown>;

      if (
        typeof name === 'string' &&
        name.trim() &&
        typeof travelMinutes === 'number' &&
        Number.isInteger(travelMinutes) &&
        travelMinutes >= 0
      ) {
        return { name: name.trim(), travelMinutes };
      }
    }
  } catch {
    // The endpoint reports all malformed bodies as an invalid location.
  }

  return undefined;
};

export const POST = async ({ request }) => {
  const location = await readLocation(request);

  if (!location) {
    return json({ error: 'invalid_location' }, { status: 400 });
  }

  return json(await configureLocation(currentLocationRepository(), location));
};
