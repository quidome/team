import { eq } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import type { TeamRepository } from '../../application/teams/team-repository';
import { createDatabase } from './database';
import { createPostgresTeamRepository } from './postgres-team-repository';
import { teams } from './schema';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  describe.skip('PostgreSQL team repository', () => {});
} else {
  describe('PostgreSQL team repository', () => {
    const name = 'U16-1 integration test';
    const database = createDatabase(databaseUrl);
    const repository: TeamRepository = createPostgresTeamRepository(database);

    beforeAll(async () => {
      await database.delete(teams).where(eq(teams.name, name));
    });

    afterAll(async () => {
      await database.delete(teams).where(eq(teams.name, name));
      await database.close();
    });

    it('persists and retrieves a configured team', async () => {
      await expect(repository.save({ isOwnTeam: true, name })).resolves.toEqual({
        isOwnTeam: true,
        name,
      });
      await expect(repository.findByName(name)).resolves.toEqual({ isOwnTeam: true, name });
    });

    it('finds only own teams via findAllOwnTeams', async () => {
      const externalName = `${name} external`;

      await database.delete(teams).where(eq(teams.name, externalName));
      await repository.save({ isOwnTeam: false, name: externalName });

      const ownTeams = await repository.findAllOwnTeams();

      expect(ownTeams.some((team) => team.name === name)).toBe(true);
      expect(ownTeams.some((team) => team.name === externalName)).toBe(false);

      await database.delete(teams).where(eq(teams.name, externalName));
    });
  });
}
