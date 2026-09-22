import { json } from '@sveltejs/kit';

import { configureCoordinatorSettings } from '$lib/application/settings/configure-coordinator-settings';
import type { CoordinatorSettings } from '$lib/application/settings/coordinator-settings-repository';
import {
  currentAuditRepository,
  currentCoordinatorSettingsRepository,
  currentSeasonRepository,
  currentTeamRepository,
} from '$lib/server/composition-root';

const readSettings = async (request: Request): Promise<CoordinatorSettings | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null) {
      const { primaryTeamName, seasonStartingYear } = payload as Record<string, unknown>;

      if (
        typeof primaryTeamName === 'string' &&
        primaryTeamName.trim() &&
        typeof seasonStartingYear === 'number' &&
        Number.isInteger(seasonStartingYear)
      ) {
        return { primaryTeamName: primaryTeamName.trim(), seasonStartingYear };
      }
    }
  } catch {
    // The endpoint reports all malformed bodies as invalid coordinator settings.
  }

  return undefined;
};

export const GET = async () => json((await currentCoordinatorSettingsRepository().get()) ?? null);

export const PUT = async ({ request }) => {
  const input = await readSettings(request);

  if (!input) {
    return json({ error: 'invalid_coordinator_settings' }, { status: 400 });
  }

  try {
    const settings = await configureCoordinatorSettings(
      currentCoordinatorSettingsRepository(),
      currentTeamRepository(),
      currentSeasonRepository(),
      input,
    );

    await currentAuditRepository().record({
      action: 'coordinator_settings_updated',
      entityId: 'singleton',
      entityType: 'coordinator_settings',
      metadata: {
        primaryTeamName: settings.primaryTeamName,
        seasonStartingYear: settings.seasonStartingYear,
      },
    });

    return json(settings);
  } catch (error) {
    if (error instanceof Error && error.message.includes('does not exist')) {
      return json({ error: error.message }, { status: 400 });
    }

    throw error;
  }
};
