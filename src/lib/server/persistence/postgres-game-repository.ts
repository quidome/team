import { alias } from 'drizzle-orm/pg-core';
import { eq } from 'drizzle-orm';

import { suggestDepartureTime } from '../../domain/game';
import type {
  GameFixture,
  GameOccurrence,
  GameRepository,
  StoredGameFixture,
  StoredGameOccurrence,
  StoredGameProgramOccurrence,
} from '../../application/games/game-repository';
import { createDatabase } from './database';
import { gameFixtures, gameOccurrences, locations, teams } from './schema';

type Database = ReturnType<typeof createDatabase>;

const homeTeams = alias(teams, 'home_teams');
const awayTeams = alias(teams, 'away_teams');

const withoutFixtureNames = (occurrence: StoredGameProgramOccurrence): StoredGameOccurrence => ({
  arrivalBufferMinutes: occurrence.arrivalBufferMinutes,
  date: occurrence.date,
  fixtureId: occurrence.fixtureId,
  id: occurrence.id,
  locationName: occurrence.locationName,
  startTime: occurrence.startTime,
  status: occurrence.status,
  suggestedDepartureTime: occurrence.suggestedDepartureTime,
  travelMinutes: occurrence.travelMinutes,
});

const findStoredOccurrence = async (
  database: Database,
  id: string,
): Promise<StoredGameProgramOccurrence | undefined> => {
  const [occurrence] = await database
    .select({
      arrivalBufferMinutes: gameOccurrences.arrivalBufferMinutes,
      awayTeamName: awayTeams.name,
      date: gameOccurrences.date,
      fixtureId: gameOccurrences.fixtureId,
      homeTeamName: homeTeams.name,
      id: gameOccurrences.id,
      locationName: locations.name,
      startTime: gameOccurrences.startTime,
      status: gameOccurrences.status,
      travelMinutes: gameOccurrences.travelMinutes,
    })
    .from(gameOccurrences)
    .innerJoin(gameFixtures, eq(gameOccurrences.fixtureId, gameFixtures.id))
    .innerJoin(homeTeams, eq(gameFixtures.homeTeamId, homeTeams.id))
    .innerJoin(awayTeams, eq(gameFixtures.awayTeamId, awayTeams.id))
    .innerJoin(locations, eq(gameOccurrences.locationId, locations.id))
    .where(eq(gameOccurrences.id, id))
    .limit(1);

  return occurrence
    ? {
        ...occurrence,
        suggestedDepartureTime: suggestDepartureTime(
          occurrence.startTime,
          occurrence.travelMinutes,
          occurrence.arrivalBufferMinutes,
        ),
      }
    : undefined;
};

export const createPostgresGameRepository = (database: Database): GameRepository => ({
  async findAllOccurrences(): Promise<StoredGameProgramOccurrence[]> {
    const occurrenceRows = await database.select({ id: gameOccurrences.id }).from(gameOccurrences);

    return (
      await Promise.all(
        occurrenceRows.map((occurrence) => findStoredOccurrence(database, occurrence.id)),
      )
    ).filter((occurrence): occurrence is StoredGameProgramOccurrence => occurrence !== undefined);
  },

  async findFixtureById(id: string): Promise<StoredGameFixture | undefined> {
    const [fixture] = await database
      .select({
        awayTeamName: awayTeams.name,
        homeTeamName: homeTeams.name,
      })
      .from(gameFixtures)
      .innerJoin(homeTeams, eq(gameFixtures.homeTeamId, homeTeams.id))
      .innerJoin(awayTeams, eq(gameFixtures.awayTeamId, awayTeams.id))
      .where(eq(gameFixtures.id, id))
      .limit(1);

    return fixture ? { fixture, id } : undefined;
  },

  async findOccurrenceById(id: string): Promise<StoredGameOccurrence | undefined> {
    const occurrence = await findStoredOccurrence(database, id);

    if (!occurrence) {
      return undefined;
    }

    return withoutFixtureNames(occurrence);
  },

  async findOccurrences(fixtureId: string): Promise<StoredGameOccurrence[]> {
    const occurrenceRows = await database
      .select({ id: gameOccurrences.id })
      .from(gameOccurrences)
      .where(eq(gameOccurrences.fixtureId, fixtureId));

    return (
      await Promise.all(
        occurrenceRows.map((occurrence) => findStoredOccurrence(database, occurrence.id)),
      )
    )
      .filter((occurrence): occurrence is StoredGameProgramOccurrence => occurrence !== undefined)
      .map(withoutFixtureNames);
  },

  async saveFixture(fixture: GameFixture): Promise<StoredGameFixture> {
    const [homeTeam] = await database
      .select({ id: teams.id })
      .from(teams)
      .where(eq(teams.name, fixture.homeTeamName))
      .limit(1);
    const [awayTeam] = await database
      .select({ id: teams.id })
      .from(teams)
      .where(eq(teams.name, fixture.awayTeamName))
      .limit(1);

    if (!homeTeam) {
      throw new Error(`Team ${fixture.homeTeamName} does not exist`);
    }

    if (!awayTeam) {
      throw new Error(`Team ${fixture.awayTeamName} does not exist`);
    }

    const [storedFixture] = await database
      .insert(gameFixtures)
      .values({ awayTeamId: awayTeam.id, homeTeamId: homeTeam.id })
      .returning({ id: gameFixtures.id });

    if (!storedFixture) {
      throw new Error('PostgreSQL did not return the stored game fixture');
    }

    return { fixture, id: storedFixture.id };
  },

  async saveOccurrence(
    fixtureId: string,
    occurrence: GameOccurrence,
  ): Promise<StoredGameOccurrence> {
    const fixture = await this.findFixtureById(fixtureId);
    const [location] = await database
      .select({ id: locations.id })
      .from(locations)
      .where(eq(locations.name, occurrence.locationName))
      .limit(1);

    if (!fixture) {
      throw new Error(`Game fixture ${fixtureId} does not exist`);
    }

    if (!location) {
      throw new Error(`Location ${occurrence.locationName} does not exist`);
    }

    const [storedOccurrence] = await database
      .insert(gameOccurrences)
      .values({
        arrivalBufferMinutes: occurrence.arrivalBufferMinutes,
        date: occurrence.date,
        fixtureId,
        locationId: location.id,
        startTime: occurrence.startTime,
        status: 'scheduled',
        travelMinutes: occurrence.travelMinutes,
      })
      .returning({ id: gameOccurrences.id });

    if (!storedOccurrence) {
      throw new Error('PostgreSQL did not return the stored game occurrence');
    }

    return {
      ...occurrence,
      fixtureId,
      id: storedOccurrence.id,
      status: 'scheduled',
      suggestedDepartureTime: suggestDepartureTime(
        occurrence.startTime,
        occurrence.travelMinutes,
        occurrence.arrivalBufferMinutes,
      ),
    };
  },

  async updateOccurrenceStatus(
    id: string,
    status: 'cancelled' | 'scheduled',
  ): Promise<StoredGameOccurrence> {
    const [updatedOccurrence] = await database
      .update(gameOccurrences)
      .set({ status })
      .where(eq(gameOccurrences.id, id))
      .returning({ id: gameOccurrences.id });

    if (!updatedOccurrence) {
      throw new Error(`Game occurrence ${id} does not exist`);
    }

    const occurrence = await this.findOccurrenceById(updatedOccurrence.id);

    if (!occurrence) {
      throw new Error(`Game occurrence ${id} could not be loaded after update`);
    }

    return occurrence;
  },
});
