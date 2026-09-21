import { eq } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import type { GameRepository } from '../../application/games/game-repository';
import { createDatabase } from './database';
import { createPostgresGameRepository } from './postgres-game-repository';
import { createPostgresSeasonRepository } from './postgres-season-repository';
import { gameFixtures, locations, seasons, teams } from './schema';

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

    describe('with an external opponent', () => {
      const opponentSeason = { endingYear: 2098, startingYear: 2097 };
      const ourTeamName = 'U16-1 opponent integration';
      const opponentLocation = { name: 'Opponent integration court', travelMinutes: 20 };
      const opponentFixture = {
        awayTeamName: 'placeholder away',
        homeTeamName: 'placeholder home',
        isHome: true,
        opponentAddress: 'Integration lane 1',
        opponentName: 'Riverside opponent integration',
        opponentTravelMinutes: 20,
        ourTeamName,
        seasonHalf: 'H1' as const,
        seasonStartingYear: opponentSeason.startingYear,
      };
      const seasonRepository = createPostgresSeasonRepository(database);
      let ourTeamId: string;
      let opponentLocationId: string;
      let opponentFixtureId: string | undefined;

      beforeAll(async () => {
        await database.delete(seasons).where(eq(seasons.startingYear, opponentSeason.startingYear));

        const [existingTeam] = await database
          .select({ id: teams.id })
          .from(teams)
          .where(eq(teams.name, ourTeamName));

        if (existingTeam) {
          await database.delete(teams).where(eq(teams.id, existingTeam.id));
        }

        const [existingLocation] = await database
          .select({ id: locations.id })
          .from(locations)
          .where(eq(locations.name, opponentLocation.name));

        if (existingLocation) {
          await database.delete(locations).where(eq(locations.id, existingLocation.id));
        }

        await seasonRepository.save(opponentSeason);

        const [team] = await database
          .insert(teams)
          .values({ name: ourTeamName })
          .returning({ id: teams.id });
        const [storedLocation] = await database
          .insert(locations)
          .values(opponentLocation)
          .returning({ id: locations.id });

        if (!team || !storedLocation) {
          throw new Error('Could not create opponent-fixture integration fixtures');
        }

        ourTeamId = team.id;
        opponentLocationId = storedLocation.id;
      });

      afterAll(async () => {
        if (opponentFixtureId) {
          await database.delete(gameFixtures).where(eq(gameFixtures.id, opponentFixtureId));
        }
        await database.delete(seasons).where(eq(seasons.startingYear, opponentSeason.startingYear));
        await database.delete(teams).where(eq(teams.id, ourTeamId));
        await database.delete(locations).where(eq(locations.id, opponentLocationId));
      });

      it('stores an opponent fixture without populating the legacy home/away team columns', async () => {
        const storedFixture = await repository.saveFixture(opponentFixture);
        opponentFixtureId = storedFixture.id;

        expect(storedFixture.fixture).toEqual({
          ...opponentFixture,
          awayTeamName: opponentFixture.opponentName,
          homeTeamName: ourTeamName,
        });

        await expect(repository.findFixtureById(storedFixture.id)).resolves.toEqual({
          fixture: {
            awayTeamName: opponentFixture.opponentName,
            homeTeamName: ourTeamName,
            isHome: true,
            opponentName: opponentFixture.opponentName,
            ourTeamName,
            seasonHalf: opponentFixture.seasonHalf,
            seasonStartingYear: opponentFixture.seasonStartingYear,
          },
          id: storedFixture.id,
        });

        const [row] = await database
          .select({ awayTeamId: gameFixtures.awayTeamId, homeTeamId: gameFixtures.homeTeamId })
          .from(gameFixtures)
          .where(eq(gameFixtures.id, storedFixture.id));

        expect(row).toEqual({ awayTeamId: null, homeTeamId: null });
      });
    });
  });
}
