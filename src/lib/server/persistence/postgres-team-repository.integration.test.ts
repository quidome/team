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
      await expect(repository.save({ name })).resolves.toEqual({ name });
      await expect(repository.findByName(name)).resolves.toEqual({ name });
    });
  });
}
