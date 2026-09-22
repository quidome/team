import type { GameImportPreview, ImportedGame } from './game-import';
import type { GameImportRepository } from './game-import-repository';
import type { GameRepository, StoredGameProgramOccurrence } from '../games/game-repository';
import type { DutyRepository } from '../duties/duty-repository';
import { normalizeTeamName, teamNamesMatch } from '../../domain/team-name';

export type GameImportConflictField = 'arrivalBufferMinutes' | 'locationName' | 'travelMinutes';
export type GameImportChoice = 'existing' | 'imported';

export interface GameImportResolution {
  fields: Partial<Record<GameImportConflictField, GameImportChoice>>;
  sourceRow: number;
}

export interface GameImportConflict {
  existingOccurrenceId: string;
  fields: {
    existingValue: string | number;
    field: GameImportConflictField;
    importedValue: string | number;
  }[];
  sourceRow: number;
}

export interface ImportedGameResult extends ImportedGame {
  fixtureId: string;
  isPrimaryTeamGame: boolean;
  merged: boolean;
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
  conflicts: GameImportConflict[];
  duplicates: DuplicateGameResult[];
  failed: FailedGameResult[];
  imported: ImportedGameResult[];
}

export interface GameImportContext {
  knownLocationTravelMinutes: Record<string, number>;
  knownTeamNames: string[];
}

export interface ImportGamesInput {
  context: GameImportContext;
  importedAt: Date;
  primaryTeamName: string;
  records: ImportedGame[];
  resolutions?: GameImportResolution[];
  sourceName: string;
  atomic?: boolean;
}

const conflictFields: GameImportConflictField[] = [
  'locationName',
  'travelMinutes',
  'arrivalBufferMinutes',
];

const fixtureKey = (homeTeamName: string, awayTeamName: string): string =>
  [normalizeTeamName(homeTeamName), normalizeTeamName(awayTeamName)].join('\u0000');

const resolveTeamName = (rawName: string, knownTeamNames: string[]): string =>
  knownTeamNames.find((known) => teamNamesMatch(known, rawName)) ?? normalizeTeamName(rawName);

const isSameIdentity = (existing: StoredGameProgramOccurrence, imported: ImportedGame) =>
  existing.date === imported.date &&
  existing.startTime === imported.startTime &&
  teamNamesMatch(existing.homeTeamName, imported.homeTeamName) &&
  teamNamesMatch(existing.awayTeamName, imported.awayTeamName);

const isSameOccurrence = (existing: StoredGameProgramOccurrence, imported: ImportedGame) =>
  isSameIdentity(existing, imported) &&
  existing.locationName === imported.locationName &&
  existing.travelMinutes === imported.travelMinutes &&
  existing.arrivalBufferMinutes === imported.arrivalBufferMinutes;

const conflictFor = (
  existing: StoredGameProgramOccurrence,
  imported: ImportedGame,
): GameImportConflict => ({
  existingOccurrenceId: existing.id,
  fields: conflictFields
    .filter((field) => existing[field] !== imported[field])
    .map((field) => ({
      existingValue: existing[field],
      field,
      importedValue: imported[field],
    })),
  sourceRow: imported.sourceRow,
});

export const findImportConflicts = async (
  games: GameRepository,
  records: ImportedGame[],
): Promise<{ conflicts: GameImportConflict[]; duplicates: DuplicateGameResult[] }> => {
  const existingOccurrences = await games.findAllOccurrences();
  const conflicts: GameImportConflict[] = [];
  const duplicates: DuplicateGameResult[] = [];

  for (const record of records) {
    const exact = existingOccurrences.find((candidate) => isSameOccurrence(candidate, record));

    if (exact) {
      duplicates.push({ existingOccurrenceId: exact.id, sourceRow: record.sourceRow });
      continue;
    }

    const identity = existingOccurrences.find((candidate) => isSameIdentity(candidate, record));

    if (identity) {
      conflicts.push(conflictFor(identity, record));
    }
  }

  return { conflicts, duplicates };
};

export const applyImportContext = (
  preview: GameImportPreview,
  context: GameImportContext,
): GameImportPreview => {
  const notices: GameImportPreview['notices'] = [];

  for (const record of preview.records) {
    const newTeamNames = [record.homeTeamName, record.awayTeamName].filter(
      (name) => !context.knownTeamNames.some((known) => teamNamesMatch(known, name)),
    );

    for (const name of newTeamNames) {
      notices.push({
        message: `"${name}" is a new team and will be created automatically.`,
        row: record.sourceRow,
      });
    }
  }

  return {
    ...preview,
    notices: [...preview.notices, ...notices],
  };
};

const occurrenceInput = (record: ImportedGame) => ({
  arrivalBufferMinutes: record.arrivalBufferMinutes,
  date: record.date,
  locationName: record.locationName,
  startTime: record.startTime,
  travelMinutes: record.travelMinutes,
});

const applyDutiesIfPresent = async (
  duties: DutyRepository,
  occurrenceId: string,
  record: ImportedGame,
): Promise<void> => {
  if (record.jurySlots === undefined && record.refereeSlots === undefined) {
    return;
  }

  await duties.configure(occurrenceId, {
    drivingSlots: 0,
    jurySlots: record.jurySlots ?? 0,
    refereeSlots: record.refereeSlots ?? 0,
  });
};

const isPrimaryTeamGame = (record: ImportedGame, primaryTeamName: string): boolean =>
  teamNamesMatch(record.homeTeamName, primaryTeamName) ||
  teamNamesMatch(record.awayTeamName, primaryTeamName);

export const importGames = async (
  games: GameRepository,
  imports: GameImportRepository,
  duties: DutyRepository,
  input: ImportGamesInput,
): Promise<GameImportResult> => {
  const existingOccurrences = await games.findAllOccurrences();
  const fixtureIds = new Map<string, string>();
  const imported: ImportedGameResult[] = [];
  const duplicates: DuplicateGameResult[] = [];
  const conflicts: GameImportConflict[] = [];
  const failed: FailedGameResult[] = [];

  for (const record of input.records) {
    const exact = existingOccurrences.find((candidate) => isSameOccurrence(candidate, record));

    if (exact) {
      duplicates.push({ existingOccurrenceId: exact.id, sourceRow: record.sourceRow });
      continue;
    }

    const identity = existingOccurrences.find((candidate) => isSameIdentity(candidate, record));
    const conflict = identity ? conflictFor(identity, record) : undefined;
    const resolution = input.resolutions?.find(
      (candidate) => candidate.sourceRow === record.sourceRow,
    );

    if (conflict && !resolution) {
      conflicts.push(conflict);
      continue;
    }

    try {
      if (identity && conflict && resolution) {
        const mergedRecord = { ...record };

        for (const field of conflictFields) {
          if (resolution.fields[field] !== 'existing') {
            continue;
          }

          if (field === 'locationName') mergedRecord.locationName = identity.locationName;
          if (field === 'travelMinutes') mergedRecord.travelMinutes = identity.travelMinutes;
          if (field === 'arrivalBufferMinutes') {
            mergedRecord.arrivalBufferMinutes = identity.arrivalBufferMinutes;
          }
        }

        const occurrence = await games.updateOccurrence(identity.id, occurrenceInput(mergedRecord));

        await applyDutiesIfPresent(duties, occurrence.id, mergedRecord);

        await imports.save({
          importedAt: input.importedAt,
          occurrenceId: occurrence.id,
          sourceName: input.sourceName,
          sourceRow: record.sourceRow,
        });

        imported.push({
          ...mergedRecord,
          fixtureId: identity.fixtureId,
          isPrimaryTeamGame: isPrimaryTeamGame(record, input.primaryTeamName),
          merged: true,
          occurrenceId: occurrence.id,
        });
        existingOccurrences[existingOccurrences.indexOf(identity)] = {
          ...occurrence,
          awayTeamName: identity.awayTeamName,
          homeTeamName: identity.homeTeamName,
        };
        continue;
      }

      const key = fixtureKey(record.homeTeamName, record.awayTeamName);
      let fixtureId = fixtureIds.get(key);

      if (!fixtureId) {
        const fixture = await games.saveFixture({
          awayTeamName: resolveTeamName(record.awayTeamName, input.context.knownTeamNames),
          homeTeamName: resolveTeamName(record.homeTeamName, input.context.knownTeamNames),
        });
        fixtureId = fixture.id;
        fixtureIds.set(key, fixtureId);
      }

      const occurrence = await games.saveOccurrence(fixtureId, occurrenceInput(record));

      await applyDutiesIfPresent(duties, occurrence.id, record);

      await imports.save({
        importedAt: input.importedAt,
        occurrenceId: occurrence.id,
        sourceName: input.sourceName,
        sourceRow: record.sourceRow,
      });

      imported.push({
        ...record,
        fixtureId,
        isPrimaryTeamGame: isPrimaryTeamGame(record, input.primaryTeamName),
        merged: false,
        occurrenceId: occurrence.id,
      });
      existingOccurrences.push({
        ...occurrence,
        awayTeamName: record.awayTeamName,
        homeTeamName: record.homeTeamName,
      });
    } catch (error) {
      if (input.atomic) {
        throw error;
      }

      failed.push({
        message: error instanceof Error ? error.message : 'The game could not be imported.',
        sourceRow: record.sourceRow,
      });
    }
  }

  return { conflicts, duplicates, failed, imported };
};
