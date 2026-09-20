import { json } from '@sveltejs/kit';

import { configureLocation } from '$lib/application/locations/configure-location';
import type { Location } from '$lib/application/locations/location-repository';
import { currentAuditRepository, currentLocationRepository } from '$lib/server/composition-root';

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

const readLocationUpdate = async (
  request: Request,
): Promise<{ currentName: string; location: Location } | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null) {
      const { currentName, name, travelMinutes } = payload as Record<string, unknown>;

      if (
        typeof currentName === 'string' &&
        currentName.trim() &&
        typeof name === 'string' &&
        name.trim() &&
        typeof travelMinutes === 'number' &&
        Number.isInteger(travelMinutes) &&
        travelMinutes >= 0
      ) {
        return {
          currentName: currentName.trim(),
          location: { name: name.trim(), travelMinutes },
        };
      }
    }
  } catch {
    // The endpoint reports malformed updates as invalid location data.
  }

  return undefined;
};

export const PUT = async ({ request }) => {
  const update = await readLocationUpdate(request);

  if (!update) {
    return json({ error: 'invalid_location' }, { status: 400 });
  }

  const location = await currentLocationRepository().updateName(
    update.currentName,
    update.location,
  );
  await currentAuditRepository().record({
    action: 'location_updated',
    entityId: location.name,
    entityType: 'location',
    metadata: { name: location.name, travelMinutes: location.travelMinutes },
  });

  return json(location);
};

export const DELETE = async ({ url }) => {
  const name = url.searchParams.get('name')?.trim();

  if (!name) {
    return json({ error: 'invalid_location' }, { status: 400 });
  }

  await currentLocationRepository().deleteByName(name);
  await currentAuditRepository().record({
    action: 'location_deleted',
    entityId: name,
    entityType: 'location',
    metadata: { name },
  });

  return json({ deleted: true });
};

export const POST = async ({ request }) => {
  const location = await readLocation(request);

  if (!location) {
    return json({ error: 'invalid_location' }, { status: 400 });
  }

  const configuredLocation = await configureLocation(currentLocationRepository(), location);

  await currentAuditRepository().record({
    action: 'location_configured',
    entityId: configuredLocation.name,
    entityType: 'location',
    metadata: { name: configuredLocation.name, travelMinutes: configuredLocation.travelMinutes },
  });

  return json(configuredLocation);
};
