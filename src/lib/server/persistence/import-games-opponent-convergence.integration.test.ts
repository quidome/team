import { and, eq } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import { importGames } from '../../application/imports/import-games';
import type { ImportedGame } from '../../application/imports/game-import';
import { createDatabase } from './database';
import { createPostgresDutyRepository } from './postgres-duty-repository';
import { createPostgresGameImportRepository } from './postgres-game-import-repository';
import { createPostgresGameRepository } from './postgres-game-repository';
import { locations, opponents, seasons, teams } from './schema';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  describe.skip('game import opponent convergence', () => {});
} else {
  describe('game import opponent convergence', () => {
    const teamName = 'Opponent convergence team';
    const locationName = 'Opponent convergence court';
    const startingYear = 2096;
    const database = createDatabase(databaseUrl);
    const games = createPostgresGameRepository(database);
    const duties = createPostgresDutyRepository(database);
    const gameImports = createPostgresGameImportRepository(database);

    const record = (date: string, awayTeamName: string, sourceRow: number): ImportedGame => ({
      arrivalBufferMinutes: 30,
      awayTeamName,
      date,
      homeTeamName: teamName,
      locationName,
      sourceRow,
      startTime: '14:30',
      travelMinutes: 20,
    });

    beforeAll(async () => {
      await database.delete(seasons).where(eq(seasons.startingYear, startingYear));
      await database.delete(teams).where(eq(teams.name, teamName));
      await database.delete(locations).where(eq(locations.name, locationName));

      await database.insert(seasons).values({ endingYear: startingYear + 1, startingYear });
      await database.insert(teams).values({ name: teamName });
      await database.insert(locations).values({ name: locationName, travelMinutes: 20 });
    });

    afterAll(async () => {
      await database.delete(seasons).where(eq(seasons.startingYear, startingYear));
      await database.delete(teams).where(eq(teams.name, teamName));
      await database.delete(locations).where(eq(locations.name, locationName));
      await database.close();
    });

    it('converges dash-formatted variants of the same opponent name onto one row', async () => {
      const context = { knownTeamNames: [teamName], season: { startingYear } };

      await importGames(games, gameImports, duties, {
        context,
        importedAt: new Date('2026-08-01T10:00:00.000Z'),
        primaryTeamName: teamName,
        records: [record('2096-09-05', 'Riverside - Dolphins M16-1', 2)],
        sourceName: 'convergence-1.csv',
      });
      await importGames(games, gameImports, duties, {
        context,
        importedAt: new Date('2026-08-02T10:00:00.000Z'),
        primaryTeamName: teamName,
        records: [record('2096-09-12', 'Riverside Dolphins M16-1', 3)],
        sourceName: 'convergence-2.csv',
      });

      const [season] = await database
        .select({ id: seasons.id })
        .from(seasons)
        .where(eq(seasons.startingYear, startingYear))
        .limit(1);
      const opponentRows = await database
        .select({ name: opponents.name })
        .from(opponents)
        .where(
          and(eq(opponents.seasonId, season!.id), eq(opponents.name, 'Riverside Dolphins M16-1')),
        );

      expect(opponentRows).toHaveLength(1);
    });
  });
}
