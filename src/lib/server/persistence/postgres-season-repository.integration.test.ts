import { eq } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import type { SeasonRepository } from '../../application/seasons/season-repository';
import { createDatabase } from './database';
import { createPostgresSeasonRepository } from './postgres-season-repository';
import { seasons } from './schema';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  describe.skip('PostgreSQL season repository', () => {});
} else {
  describe('PostgreSQL season repository', () => {
    const startingYear = 2098;
    const database = createDatabase(databaseUrl);
    const repository: SeasonRepository = createPostgresSeasonRepository(database);

    beforeAll(async () => {
      await database.delete(seasons).where(eq(seasons.startingYear, startingYear));
    });

    afterAll(async () => {
      await database.delete(seasons).where(eq(seasons.startingYear, startingYear));
      await database.close();
    });

    it('persists and retrieves a configured season', async () => {
      await expect(repository.save({ endingYear: 2099, startingYear })).resolves.toEqual({
        endingYear: 2099,
        startingYear,
      });
      await expect(repository.findByStartingYear(startingYear)).resolves.toEqual({
        endingYear: 2099,
        startingYear,
      });
    });
  });
}
