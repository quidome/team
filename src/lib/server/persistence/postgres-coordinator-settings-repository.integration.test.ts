import { eq } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import type { CoordinatorSettingsRepository } from '../../application/settings/coordinator-settings-repository';
import { createDatabase } from './database';
import { createPostgresCoordinatorSettingsRepository } from './postgres-coordinator-settings-repository';
import { coordinatorSettings, seasons, teams } from './schema';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  describe.skip('PostgreSQL coordinator settings repository', () => {});
} else {
  describe('PostgreSQL coordinator settings repository', () => {
    const teamName = 'Coordinator settings integration team';
    const startingYear = 2091;
    const database = createDatabase(databaseUrl);
    const repository: CoordinatorSettingsRepository =
      createPostgresCoordinatorSettingsRepository(database);

    beforeAll(async () => {
      await database.delete(coordinatorSettings).where(eq(coordinatorSettings.id, 'singleton'));
      await database.delete(teams).where(eq(teams.name, teamName));
      await database.delete(seasons).where(eq(seasons.startingYear, startingYear));

      await database.insert(teams).values({ name: teamName });
      await database.insert(seasons).values({ endingYear: startingYear + 1, startingYear });
    });

    afterAll(async () => {
      await database.delete(coordinatorSettings).where(eq(coordinatorSettings.id, 'singleton'));
      await database.delete(teams).where(eq(teams.name, teamName));
      await database.delete(seasons).where(eq(seasons.startingYear, startingYear));
      await database.close();
    });

    it('persists and retrieves the singleton settings row', async () => {
      const settings = { primaryTeamName: teamName, seasonStartingYear: startingYear };

      await expect(repository.save(settings)).resolves.toEqual(settings);
      await expect(repository.get()).resolves.toEqual(settings);
    });

    it('rejects an unknown team', async () => {
      await expect(
        repository.save({ primaryTeamName: 'Unknown team', seasonStartingYear: startingYear }),
      ).rejects.toThrow('Team Unknown team does not exist');
    });

    it('rejects an unknown season', async () => {
      await expect(
        repository.save({ primaryTeamName: teamName, seasonStartingYear: 1900 }),
      ).rejects.toThrow('Season 1900 does not exist');
    });
  });
}
