import { eq } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import type { GameRepository } from '../../application/games/game-repository';
import { createDatabase } from './database';
import { createPostgresGameRepository } from './postgres-game-repository';
import { gameFixtures, locations, teams } from './schema';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  describe.skip('PostgreSQL game repository', () => {});
} else {
  describe('PostgreSQL game repository', () => {
    const fixture = {
      awayTeamName: 'U18-1 game integration',
      homeTeamName: 'U16-1 game integration',
    };
    const location = { name: 'Game integration court', travelMinutes: 0 };
    const database = createDatabase(databaseUrl);
    const repository: GameRepository = createPostgresGameRepository(database);
    let awayTeamId: string;
    let homeTeamId: string;
    let fixtureId: string | undefined;
    let locationId: string;

    beforeAll(async () => {
      const [existingAwayTeam] = await database
        .select({ id: teams.id })
        .from(teams)
        .where(eq(teams.name, fixture.awayTeamName));
      const [existingHomeTeam] = await database
        .select({ id: teams.id })
        .from(teams)
        .where(eq(teams.name, fixture.homeTeamName));
      const [existingLocation] = await database
        .select({ id: locations.id })
        .from(locations)
        .where(eq(locations.name, location.name));

      if (existingAwayTeam && existingHomeTeam) {
        await database.delete(gameFixtures).where(eq(gameFixtures.awayTeamId, existingAwayTeam.id));
      }
      if (existingAwayTeam) {
        await database.delete(teams).where(eq(teams.id, existingAwayTeam.id));
      }
      if (existingHomeTeam) {
        await database.delete(teams).where(eq(teams.id, existingHomeTeam.id));
      }
      if (existingLocation) {
        await database.delete(locations).where(eq(locations.id, existingLocation.id));
      }

      const [awayTeam] = await database
        .insert(teams)
        .values({ name: fixture.awayTeamName })
        .returning({ id: teams.id });
      const [homeTeam] = await database
        .insert(teams)
        .values({ name: fixture.homeTeamName })
        .returning({ id: teams.id });
      const [storedLocation] = await database
        .insert(locations)
        .values(location)
        .returning({ id: locations.id });

      if (!awayTeam || !homeTeam || !storedLocation) {
        throw new Error('Could not create game integration fixtures');
      }

      awayTeamId = awayTeam.id;
      homeTeamId = homeTeam.id;
      locationId = storedLocation.id;
    });

    afterAll(async () => {
      if (fixtureId) {
        await database.delete(gameFixtures).where(eq(gameFixtures.id, fixtureId));
      }
      await database.delete(teams).where(eq(teams.id, awayTeamId));
      await database.delete(teams).where(eq(teams.id, homeTeamId));
      await database.delete(locations).where(eq(locations.id, locationId));
      await database.close();
    });

    it('persists a stable fixture and a scheduled occurrence', async () => {
      const storedFixture = await repository.saveFixture(fixture);
      fixtureId = storedFixture.id;
      const occurrence = await repository.saveOccurrence(storedFixture.id, {
        arrivalBufferMinutes: 30,
        date: '2095-09-05',
        locationName: location.name,
        startTime: '14:30',
        travelMinutes: 0,
      });

      await expect(repository.findFixtureById(storedFixture.id)).resolves.toEqual(storedFixture);
      await expect(repository.findOccurrences(storedFixture.id)).resolves.toEqual([occurrence]);
    });
  });
}
