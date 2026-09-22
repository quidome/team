import { eq } from 'drizzle-orm';
import { alias } from 'drizzle-orm/pg-core';

import { suggestDepartureTime } from '../../domain/game';
import type {
  GameFixture,
  GameOccurrence,
  GameRepository,
  StoredGameFixture,
  StoredGameOccurrence,
  StoredGameProgramOccurrence,
} from '../../application/games/game-repository';
import type { DatabaseConnection } from './database';
import { gameFixtures, gameOccurrences, locations, teams } from './schema';

const homeTeams = alias(teams, 'home_teams');
const awayTeams = alias(teams, 'away_teams');

type ProgramOccurrenceRow = {
  arrivalBufferMinutes: number;
  awayTeamName: string;
  date: string;
  fixtureId: string;
  homeTeamName: string;
  id: string;
  locationName: string;
  startTime: string;
  status: 'cancelled' | 'scheduled';
  travelMinutes: number;
};

const toProgramOccurrence = (row: ProgramOccurrenceRow): StoredGameProgramOccurrence => ({
  arrivalBufferMinutes: row.arrivalBufferMinutes,
  awayTeamName: row.awayTeamName,
  date: row.date,
  fixtureId: row.fixtureId,
  homeTeamName: row.homeTeamName,
  id: row.id,
  locationName: row.locationName,
  startTime: row.startTime,
  status: row.status,
  suggestedDepartureTime: suggestDepartureTime(
    row.startTime,
    row.travelMinutes,
    row.arrivalBufferMinutes,
  ),
  travelMinutes: row.travelMinutes,
});

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
  database: DatabaseConnection,
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

  return occurrence ? toProgramOccurrence(occurrence) : undefined;
};

const findOrCreateTeamId = async (database: DatabaseConnection, name: string): Promise<string> => {
  const [existing] = await database
    .select({ id: teams.id })
    .from(teams)
    .where(eq(teams.name, name))
    .limit(1);

  if (existing) {
    return existing.id;
  }

  const [created] = await database
    .insert(teams)
    .values({ isOwnTeam: false, name })
    .returning({ id: teams.id });

  if (!created) {
    throw new Error(`Team ${name} could not be stored`);
  }

  return created.id;
};

export const createPostgresGameRepository = (database: DatabaseConnection): GameRepository => ({
  async findAllOccurrences(): Promise<StoredGameProgramOccurrence[]> {
    const occurrenceRows = await database.select({ id: gameOccurrences.id }).from(gameOccurrences);

    return (
      await Promise.all(
        occurrenceRows.map((occurrence) => findStoredOccurrence(database, occurrence.id)),
      )
    ).filter((occurrence): occurrence is StoredGameProgramOccurrence => occurrence !== undefined);
  },

  async findFixtureById(id: string): Promise<StoredGameFixture | undefined> {
    const [row] = await database
      .select({
        awayTeamName: awayTeams.name,
        homeTeamName: homeTeams.name,
      })
      .from(gameFixtures)
      .innerJoin(homeTeams, eq(gameFixtures.homeTeamId, homeTeams.id))
      .innerJoin(awayTeams, eq(gameFixtures.awayTeamId, awayTeams.id))
      .where(eq(gameFixtures.id, id))
      .limit(1);

    return row ? { fixture: row, id } : undefined;
  },

  async findOccurrenceById(id: string): Promise<StoredGameOccurrence | undefined> {
    const occurrence = await findStoredOccurrence(database, id);

    return occurrence ? withoutFixtureNames(occurrence) : undefined;
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
    const homeTeamId = await findOrCreateTeamId(database, fixture.homeTeamName);
    const awayTeamId = await findOrCreateTeamId(database, fixture.awayTeamName);

    const [storedFixture] = await database
      .insert(gameFixtures)
      .values({ awayTeamId, homeTeamId })
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

  async updateOccurrence(id: string, occurrence: GameOccurrence): Promise<StoredGameOccurrence> {
    const [location] = await database
      .select({ id: locations.id })
      .from(locations)
      .where(eq(locations.name, occurrence.locationName))
      .limit(1);

    if (!location) {
      throw new Error(`Location ${occurrence.locationName} does not exist`);
    }

    const [updatedOccurrence] = await database
      .update(gameOccurrences)
      .set({
        arrivalBufferMinutes: occurrence.arrivalBufferMinutes,
        date: occurrence.date,
        locationId: location.id,
        startTime: occurrence.startTime,
        travelMinutes: occurrence.travelMinutes,
      })
      .where(eq(gameOccurrences.id, id))
      .returning({ id: gameOccurrences.id });

    if (!updatedOccurrence) {
      throw new Error(`Game occurrence ${id} does not exist`);
    }

    const updated = await this.findOccurrenceById(updatedOccurrence.id);

    if (!updated) {
      throw new Error(`Game occurrence ${id} could not be loaded after update`);
    }

    return updated;
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
