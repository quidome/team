import type { ImportedGame } from './game-import';
import type { GameImportRepository } from './game-import-repository';
import type { GameRepository, StoredGameProgramOccurrence } from '../games/game-repository';

export interface ImportedGameResult extends ImportedGame {
  fixtureId: string;
  isPrimaryTeamGame: boolean;
  occurrenceId: string;
}

export interface DuplicateGameResult {
  existingOccurrenceId: string;
  sourceRow: number;
}

export interface FailedGameResult {
  message: string;
  sourceRow: number;
}

export interface GameImportResult {
  duplicates: DuplicateGameResult[];
  failed: FailedGameResult[];
  imported: ImportedGameResult[];
}

interface ImportGamesInput {
  importedAt: Date;
  primaryTeamName: string;
  records: ImportedGame[];
  sourceName: string;
}

const fixtureKey = (homeTeamName: string, awayTeamName: string) =>
  `${homeTeamName}\u0000${awayTeamName}`;

const isSameOccurrence = (existing: StoredGameProgramOccurrence, imported: ImportedGame) =>
  existing.date === imported.date &&
  existing.startTime === imported.startTime &&
  existing.locationName === imported.locationName &&
  existing.travelMinutes === imported.travelMinutes &&
  existing.arrivalBufferMinutes === imported.arrivalBufferMinutes &&
  existing.homeTeamName === imported.homeTeamName &&
  existing.awayTeamName === imported.awayTeamName;

export const importGames = async (
  games: GameRepository,
  imports: GameImportRepository,
  input: ImportGamesInput,
): Promise<GameImportResult> => {
  const existingOccurrences = await games.findAllOccurrences();
  const fixtureIds = new Map<string, string>();
  const imported: ImportedGameResult[] = [];
  const duplicates: DuplicateGameResult[] = [];
  const failed: FailedGameResult[] = [];

  for (const record of input.records) {
    const duplicate = existingOccurrences.find((candidate) => isSameOccurrence(candidate, record));

    if (duplicate) {
      duplicates.push({ existingOccurrenceId: duplicate.id, sourceRow: record.sourceRow });
      continue;
    }

    try {
      const key = fixtureKey(record.homeTeamName, record.awayTeamName);
      let fixtureId = fixtureIds.get(key);

      if (!fixtureId) {
        const fixture = await games.saveFixture({
          awayTeamName: record.awayTeamName,
          homeTeamName: record.homeTeamName,
        });
        fixtureId = fixture.id;
        fixtureIds.set(key, fixtureId);
      }

      const occurrence = await games.saveOccurrence(fixtureId, {
        arrivalBufferMinutes: record.arrivalBufferMinutes,
        date: record.date,
        locationName: record.locationName,
        startTime: record.startTime,
        travelMinutes: record.travelMinutes,
      });

      await imports.save({
        importedAt: input.importedAt,
        occurrenceId: occurrence.id,
        sourceName: input.sourceName,
        sourceRow: record.sourceRow,
      });

      const result = {
        ...record,
        fixtureId,
        isPrimaryTeamGame:
          record.homeTeamName === input.primaryTeamName ||
          record.awayTeamName === input.primaryTeamName,
        occurrenceId: occurrence.id,
      };
      imported.push(result);
      existingOccurrences.push({
        ...occurrence,
        awayTeamName: record.awayTeamName,
        homeTeamName: record.homeTeamName,
      });
    } catch (error) {
      failed.push({
        message: error instanceof Error ? error.message : 'The game could not be imported.',
        sourceRow: record.sourceRow,
      });
    }
  }

  return { duplicates, failed, imported };
};
