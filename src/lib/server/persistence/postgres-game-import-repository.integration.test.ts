import { eq } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import { createDatabase } from './database';
import { createPostgresGameImportRepository } from './postgres-game-import-repository';
import { createPostgresGameRepository } from './postgres-game-repository';
import { locations, teams } from './schema';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  describe.skip('PostgreSQL game import repository', () => {});
} else {
  describe('PostgreSQL game import repository', () => {
    const database = createDatabase(databaseUrl);
    const homeTeamName = 'U16-1 provenance integration';
    const awayTeamName = 'U18-1 provenance integration';
    const locationName = 'Provenance integration court';
    const games = createPostgresGameRepository(database);
    const imports = createPostgresGameImportRepository(database);
    let homeTeamId: string;
    let awayTeamId: string;
    let locationId: string;
    let occurrenceId: string;

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
        throw new Error('Could not create provenance integration fixtures');
      }

      homeTeamId = homeTeam.id;
      awayTeamId = awayTeam.id;
      locationId = location.id;

      const fixture = await games.saveFixture({ awayTeamName, homeTeamName });
      const occurrence = await games.saveOccurrence(fixture.id, {
        arrivalBufferMinutes: 30,
        date: '2095-02-10',
        locationName,
        startTime: '14:30',
        travelMinutes: 0,
      });

      occurrenceId = occurrence.id;
    });

    afterAll(async () => {
      await database.delete(teams).where(eq(teams.id, homeTeamId));
      await database.delete(teams).where(eq(teams.id, awayTeamId));
      await database.delete(locations).where(eq(locations.id, locationId));
      await database.close();
    });

    it('upserts by occurrence instead of violating the occurrence unique constraint on a repeat save', async () => {
      const first = await imports.save({
        importedAt: new Date('2095-01-01T10:00:00.000Z'),
        occurrenceId,
        sourceName: 'provenance-integration-1.csv',
        sourceRow: 2,
      });

      const second = await imports.save({
        importedAt: new Date('2095-01-05T10:00:00.000Z'),
        occurrenceId,
        sourceName: 'provenance-integration-2.csv',
        sourceRow: 5,
      });

      expect(second).toMatchObject({
        id: first.id,
        occurrenceId,
        sourceName: 'provenance-integration-2.csv',
        sourceRow: 5,
      });
    });
  });
}
