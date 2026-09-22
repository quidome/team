import { eq } from 'drizzle-orm';

import type {
  CoordinatorSettings,
  CoordinatorSettingsRepository,
} from '../../application/settings/coordinator-settings-repository';
import { createDatabase } from './database';
import { coordinatorSettings, seasons, teams } from './schema';

type Database = ReturnType<typeof createDatabase>;

const settingsId = 'singleton';

export const createPostgresCoordinatorSettingsRepository = (
  database: Database,
): CoordinatorSettingsRepository => ({
  async get(): Promise<CoordinatorSettings | undefined> {
    const [row] = await database
      .select({
        primaryTeamName: teams.name,
        seasonStartingYear: seasons.startingYear,
      })
      .from(coordinatorSettings)
      .innerJoin(teams, eq(coordinatorSettings.primaryTeamId, teams.id))
      .innerJoin(seasons, eq(coordinatorSettings.seasonId, seasons.id))
      .where(eq(coordinatorSettings.id, settingsId))
      .limit(1);

    return row;
  },

  async save(settings: CoordinatorSettings): Promise<CoordinatorSettings> {
    const [team] = await database
      .select({ id: teams.id })
      .from(teams)
      .where(eq(teams.name, settings.primaryTeamName))
      .limit(1);
    const [season] = await database
      .select({ id: seasons.id })
      .from(seasons)
      .where(eq(seasons.startingYear, settings.seasonStartingYear))
      .limit(1);

    if (!team) {
      throw new Error(`Team ${settings.primaryTeamName} does not exist`);
    }

    if (!season) {
      throw new Error(`Season ${settings.seasonStartingYear} does not exist`);
    }

    await database
      .insert(coordinatorSettings)
      .values({ id: settingsId, primaryTeamId: team.id, seasonId: season.id })
      .onConflictDoUpdate({
        set: { primaryTeamId: team.id, seasonId: season.id, updatedAt: new Date() },
        target: coordinatorSettings.id,
      });

    return settings;
  },
});
