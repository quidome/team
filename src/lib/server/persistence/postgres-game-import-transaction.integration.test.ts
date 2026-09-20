import { and, eq } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import { createDatabase } from './database';
import { createPostgresGameImportRepository } from './postgres-game-import-repository';
import { createPostgresGameRepository } from './postgres-game-repository';
import { gameFixtures, locations, teams } from './schema';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  describe.skip('PostgreSQL game import transaction', () => {});
} else {
  describe('PostgreSQL game import transaction', () => {
    const database = createDatabase(databaseUrl);
    const homeTeamName = 'U16-1 transaction integration';
    const awayTeamName = 'U18-1 transaction integration';
    const locationName = 'Transaction integration court';
    let homeTeamId: string;
    let awayTeamId: string;
    let locationId: string;

    beforeAll(async () => {
      const [homeTeam] = await database
        .insert(teams)
        .values({ name: homeTeamName })
        .returning({ id: teams.id });
      const [awayTeam] = await database
        .insert(teams)
        .values({ name: awayTeamName })
        .returning({ id: teams.id });
      const [location] = await database
        .insert(locations)
        .values({ name: locationName, travelMinutes: 0 })
        .returning({ id: locations.id });

      if (!homeTeam || !awayTeam || !location) {
        throw new Error('Could not create transaction integration fixtures');
      }

      homeTeamId = homeTeam.id;
      awayTeamId = awayTeam.id;
      locationId = location.id;
    });

    afterAll(async () => {
      await database.delete(teams).where(eq(teams.id, homeTeamId));
      await database.delete(teams).where(eq(teams.id, awayTeamId));
      await database.delete(locations).where(eq(locations.id, locationId));
      await database.close();
    });

    it('rolls back game and provenance writes together', async () => {
      await expect(
        database.transaction(async (transaction) => {
          const games = createPostgresGameRepository(transaction);
          const imports = createPostgresGameImportRepository(transaction);
          const fixture = await games.saveFixture({
            awayTeamName,
            homeTeamName,
          });
          const occurrence = await games.saveOccurrence(fixture.id, {
            arrivalBufferMinutes: 30,
            date: '2095-01-10',
            locationName,
            startTime: '14:30',
            travelMinutes: 0,
          });

          await imports.save({
            importedAt: new Date('2095-01-01T10:00:00.000Z'),
            occurrenceId: occurrence.id,
            sourceName: 'transaction-integration.csv',
            sourceRow: 2,
          });

          throw new Error('force transaction rollback');
        }),
      ).rejects.toThrow('force transaction rollback');

      await expect(
        database
          .select({ id: gameFixtures.id })
          .from(gameFixtures)
          .where(
            and(eq(gameFixtures.homeTeamId, homeTeamId), eq(gameFixtures.awayTeamId, awayTeamId)),
          ),
      ).resolves.toEqual([]);
    });
  });
}
