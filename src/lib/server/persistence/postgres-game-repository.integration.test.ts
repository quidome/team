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
        .values({ isOwnTeam: true, name: fixture.awayTeamName })
        .returning({ id: teams.id });
      const [homeTeam] = await database
        .insert(teams)
        .values({ isOwnTeam: true, name: fixture.homeTeamName })
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
      await expect(repository.findAllOccurrences()).resolves.toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            awayTeamName: fixture.awayTeamName,
            homeTeamName: fixture.homeTeamName,
            id: occurrence.id,
          }),
        ]),
      );
    });

    describe('creating a team on the fly', () => {
      const knownTeamName = 'U16-1 on-the-fly integration';
      const newOpponentName = 'Riverside on-the-fly integration';
      const bothNewHomeName = 'Woodpeckers on-the-fly integration';
      const bothNewAwayName = 'Archipel on-the-fly integration';
      let knownTeamId: string;
      let createdFixtureIds: string[];

      beforeAll(async () => {
        for (const name of [knownTeamName, newOpponentName, bothNewHomeName, bothNewAwayName]) {
          await database.delete(teams).where(eq(teams.name, name));
        }

        const [team] = await database
          .insert(teams)
          .values({ isOwnTeam: true, name: knownTeamName })
          .returning({ id: teams.id });

        if (!team) {
          throw new Error('Could not create on-the-fly integration fixture');
        }

        knownTeamId = team.id;
        createdFixtureIds = [];
      });

      afterAll(async () => {
        for (const id of createdFixtureIds) {
          await database.delete(gameFixtures).where(eq(gameFixtures.id, id));
        }
        for (const name of [knownTeamName, newOpponentName, bothNewHomeName, bothNewAwayName]) {
          await database.delete(teams).where(eq(teams.name, name));
        }
      });

      it('creates a new external team when only one side is recognized', async () => {
        const storedFixture = await repository.saveFixture({
          awayTeamName: newOpponentName,
          homeTeamName: knownTeamName,
        });
        createdFixtureIds.push(storedFixture.id);

        expect(storedFixture.fixture).toEqual({
          awayTeamName: newOpponentName,
          homeTeamName: knownTeamName,
        });
        await expect(repository.findFixtureById(storedFixture.id)).resolves.toEqual(storedFixture);

        const [opponentTeam] = await database
          .select({ id: teams.id, isOwnTeam: teams.isOwnTeam })
          .from(teams)
          .where(eq(teams.name, newOpponentName));

        expect(opponentTeam?.isOwnTeam).toBe(false);

        const [row] = await database
          .select({ awayTeamId: gameFixtures.awayTeamId, homeTeamId: gameFixtures.homeTeamId })
          .from(gameFixtures)
          .where(eq(gameFixtures.id, storedFixture.id));

        expect(row).toEqual({ awayTeamId: opponentTeam?.id, homeTeamId: knownTeamId });
      });

      it('creates two new external teams for a duty-only game where neither side is recognized', async () => {
        const storedFixture = await repository.saveFixture({
          awayTeamName: bothNewAwayName,
          homeTeamName: bothNewHomeName,
        });
        createdFixtureIds.push(storedFixture.id);

        expect(storedFixture.fixture).toEqual({
          awayTeamName: bothNewAwayName,
          homeTeamName: bothNewHomeName,
        });
        await expect(repository.findFixtureById(storedFixture.id)).resolves.toEqual(storedFixture);

        const createdTeams = await database
          .select({ isOwnTeam: teams.isOwnTeam, name: teams.name })
          .from(teams)
          .where(eq(teams.name, bothNewHomeName));

        expect(createdTeams).toEqual([{ isOwnTeam: false, name: bothNewHomeName }]);
      });
    });
  });
}
