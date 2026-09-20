import { and, eq } from 'drizzle-orm';
import { alias } from 'drizzle-orm/pg-core';

import { suggestDepartureTime } from '../../domain/game';
import type {
  GameFixture,
  GameOccurrence,
  GameRepository,
  SeasonHalf,
  StoredGameFixture,
  StoredGameOccurrence,
  StoredGameProgramOccurrence,
} from '../../application/games/game-repository';
import type { DatabaseConnection } from './database';
import {
  gameFixtures,
  gameOccurrences,
  locations,
  opponentSeasonHalves,
  opponents,
  seasonHalves,
  seasons,
  teams,
} from './schema';

const homeTeams = alias(teams, 'home_teams');
const awayTeams = alias(teams, 'away_teams');
const ourTeams = alias(teams, 'our_teams');

type ProgramOccurrenceRow = {
  arrivalBufferMinutes: number;
  awayTeamName: string | null;
  date: string;
  fixtureId: string;
  homeTeamName: string | null;
  id: string;
  isHome: boolean;
  locationName: string;
  opponentName: string | null;
  ourTeamName: string | null;
  seasonHalf: SeasonHalf | null;
  startTime: string;
  status: 'cancelled' | 'scheduled';
  travelMinutes: number;
};

type FixtureRow = {
  awayTeamName: string | null;
  homeTeamName: string | null;
  isHome: boolean;
  opponentName: string | null;
  ourTeamName: string | null;
  seasonHalf: SeasonHalf | null;
  seasonStartingYear: number | null;
};

const toFixture = (row: FixtureRow): GameFixture | undefined => {
  const homeTeamName =
    row.opponentName && row.ourTeamName
      ? row.isHome
        ? row.ourTeamName
        : row.opponentName
      : row.homeTeamName;
  const awayTeamName =
    row.opponentName && row.ourTeamName
      ? row.isHome
        ? row.opponentName
        : row.ourTeamName
      : row.awayTeamName;

  if (!homeTeamName || !awayTeamName) {
    return undefined;
  }

  return {
    awayTeamName,
    homeTeamName,
    ...(row.opponentName ? { opponentName: row.opponentName } : {}),
    ...(row.ourTeamName ? { ourTeamName: row.ourTeamName } : {}),
    ...(row.seasonHalf ? { seasonHalf: row.seasonHalf } : {}),
    ...(row.seasonStartingYear !== null ? { seasonStartingYear: row.seasonStartingYear } : {}),
    isHome: row.isHome,
  };
};

const toProgramOccurrence = (
  row: ProgramOccurrenceRow,
): StoredGameProgramOccurrence | undefined => {
  const homeTeamName =
    row.opponentName && row.ourTeamName
      ? row.isHome
        ? row.ourTeamName
        : row.opponentName
      : row.homeTeamName;
  const awayTeamName =
    row.opponentName && row.ourTeamName
      ? row.isHome
        ? row.opponentName
        : row.ourTeamName
      : row.awayTeamName;

  if (!homeTeamName || !awayTeamName) {
    return undefined;
  }

  return {
    arrivalBufferMinutes: row.arrivalBufferMinutes,
    awayTeamName,
    ...(row.seasonHalf ? { seasonHalf: row.seasonHalf } : {}),
    date: row.date,
    fixtureId: row.fixtureId,
    homeTeamName,
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
  };
};

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
      isHome: gameFixtures.isHome,
      locationName: locations.name,
      opponentName: opponents.name,
      ourTeamName: ourTeams.name,
      seasonHalf: seasonHalves.half,
      startTime: gameOccurrences.startTime,
      status: gameOccurrences.status,
      travelMinutes: gameOccurrences.travelMinutes,
    })
    .from(gameOccurrences)
    .innerJoin(gameFixtures, eq(gameOccurrences.fixtureId, gameFixtures.id))
    .leftJoin(homeTeams, eq(gameFixtures.homeTeamId, homeTeams.id))
    .leftJoin(awayTeams, eq(gameFixtures.awayTeamId, awayTeams.id))
    .leftJoin(ourTeams, eq(gameFixtures.ourTeamId, ourTeams.id))
    .leftJoin(opponents, eq(gameFixtures.opponentId, opponents.id))
    .leftJoin(seasonHalves, eq(gameFixtures.seasonHalfId, seasonHalves.id))
    .innerJoin(locations, eq(gameOccurrences.locationId, locations.id))
    .where(eq(gameOccurrences.id, id))
    .limit(1);

  return occurrence ? toProgramOccurrence(occurrence) : undefined;
};

const saveOpponentFixture = async (
  database: DatabaseConnection,
  fixture: GameFixture,
): Promise<StoredGameFixture> => {
  if (
    !fixture.opponentName ||
    !fixture.ourTeamName ||
    fixture.seasonStartingYear === undefined ||
    !fixture.seasonHalf
  ) {
    throw new Error('Opponent fixtures require a team, season, half, and opponent');
  }

  const [ourTeam] = await database
    .select({ id: teams.id })
    .from(teams)
    .where(eq(teams.name, fixture.ourTeamName))
    .limit(1);
  const [season] = await database
    .select({ id: seasons.id })
    .from(seasons)
    .where(eq(seasons.startingYear, fixture.seasonStartingYear))
    .limit(1);

  if (!ourTeam) {
    throw new Error(`Team ${fixture.ourTeamName} does not exist`);
  }

  if (!season) {
    throw new Error(`Season ${fixture.seasonStartingYear} does not exist`);
  }

  let [half] = await database
    .select({ id: seasonHalves.id })
    .from(seasonHalves)
    .where(and(eq(seasonHalves.seasonId, season.id), eq(seasonHalves.half, fixture.seasonHalf)))
    .limit(1);

  if (!half) {
    [half] = await database
      .insert(seasonHalves)
      .values({ half: fixture.seasonHalf, seasonId: season.id })
      .returning({ id: seasonHalves.id });
  }

  if (!half) {
    throw new Error('Season half could not be stored');
  }

  let [opponent] = await database
    .select({ id: opponents.id })
    .from(opponents)
    .where(and(eq(opponents.name, fixture.opponentName), eq(opponents.seasonId, season.id)))
    .limit(1);

  if (!opponent) {
    [opponent] = await database
      .insert(opponents)
      .values({
        address: fixture.opponentAddress,
        name: fixture.opponentName,
        seasonId: season.id,
        travelMinutes: fixture.opponentTravelMinutes ?? 0,
      })
      .returning({ id: opponents.id });
  }

  if (!opponent) {
    throw new Error('Opponent could not be stored');
  }

  if (fixture.opponentAddress !== undefined || fixture.opponentTravelMinutes !== undefined) {
    await database
      .update(opponents)
      .set({
        ...(fixture.opponentAddress === undefined ? {} : { address: fixture.opponentAddress }),
        ...(fixture.opponentTravelMinutes === undefined
          ? {}
          : { travelMinutes: fixture.opponentTravelMinutes }),
      })
      .where(eq(opponents.id, opponent.id));
  }

  const [existingHalf] = await database
    .select({ id: opponentSeasonHalves.id })
    .from(opponentSeasonHalves)
    .where(
      and(
        eq(opponentSeasonHalves.halfId, half.id),
        eq(opponentSeasonHalves.opponentId, opponent.id),
      ),
    )
    .limit(1);

  if (!existingHalf) {
    await database.insert(opponentSeasonHalves).values({
      halfId: half.id,
      opponentId: opponent.id,
    });
  }

  const isHome = fixture.isHome ?? true;
  const [storedFixture] = await database
    .insert(gameFixtures)
    .values({
      awayTeamId: isHome ? null : ourTeam.id,
      homeTeamId: isHome ? ourTeam.id : null,
      isHome,
      opponentId: opponent.id,
      ourTeamId: ourTeam.id,
      seasonHalfId: half.id,
    })
    .returning({ id: gameFixtures.id });

  if (!storedFixture) {
    throw new Error('PostgreSQL did not return the stored game fixture');
  }

  return {
    fixture: {
      ...fixture,
      awayTeamName: isHome ? fixture.opponentName : fixture.ourTeamName,
      homeTeamName: isHome ? fixture.ourTeamName : fixture.opponentName,
    },
    id: storedFixture.id,
  };
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
        isHome: gameFixtures.isHome,
        opponentName: opponents.name,
        ourTeamName: ourTeams.name,
        seasonHalf: seasonHalves.half,
        seasonStartingYear: seasons.startingYear,
      })
      .from(gameFixtures)
      .leftJoin(homeTeams, eq(gameFixtures.homeTeamId, homeTeams.id))
      .leftJoin(awayTeams, eq(gameFixtures.awayTeamId, awayTeams.id))
      .leftJoin(ourTeams, eq(gameFixtures.ourTeamId, ourTeams.id))
      .leftJoin(opponents, eq(gameFixtures.opponentId, opponents.id))
      .leftJoin(seasonHalves, eq(gameFixtures.seasonHalfId, seasonHalves.id))
      .leftJoin(seasons, eq(seasonHalves.seasonId, seasons.id))
      .where(eq(gameFixtures.id, id))
      .limit(1);

    if (!row) {
      return undefined;
    }

    const fixture = toFixture(row);

    return fixture ? { fixture, id } : undefined;
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
    if (fixture.opponentName) {
      return saveOpponentFixture(database, fixture);
    }

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
