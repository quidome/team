import { json } from '@sveltejs/kit';

import { previewGameImport, type GameImportMapping } from '$lib/application/imports/game-import';
import {
  findImportConflicts,
  importGames,
  type GameImportChoice,
  type GameImportConflictField,
  type GameImportResolution,
} from '$lib/application/imports/import-games';
import {
  readGameImportFile,
  type GameImportFileEncoding,
  type GameImportUpload,
} from '$lib/server/imports/game-import-upload';
import { currentGameRepository, withCurrentImportTransaction } from '$lib/server/composition-root';

interface ImportRequest {
  mapping: GameImportMapping;
  primaryTeamName: string;
  resolutions: GameImportResolution[];
  sourceName: string;
  upload: GameImportUpload;
}

const conflictFields = new Set<GameImportConflictField>([
  'arrivalBufferMinutes',
  'locationName',
  'travelMinutes',
]);

const readResolutions = (value: unknown): GameImportResolution[] | undefined => {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    return undefined;
  }

  const resolutions: GameImportResolution[] = [];

  for (const candidate of value) {
    if (typeof candidate !== 'object' || candidate === null) {
      return undefined;
    }

    const { fields, sourceRow } = candidate as Record<string, unknown>;

    if (
      typeof sourceRow !== 'number' ||
      !Number.isInteger(sourceRow) ||
      sourceRow < 2 ||
      typeof fields !== 'object' ||
      fields === null
    ) {
      return undefined;
    }

    const choices: Partial<Record<GameImportConflictField, GameImportChoice>> = {};

    for (const [field, choice] of Object.entries(fields)) {
      if (
        !conflictFields.has(field as GameImportConflictField) ||
        (choice !== 'existing' && choice !== 'imported')
      ) {
        return undefined;
      }

      choices[field as GameImportConflictField] = choice;
    }

    resolutions.push({ fields: choices, sourceRow });
  }

  return resolutions;
};

const readRequest = async (request: Request): Promise<ImportRequest | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload !== 'object' || payload === null) {
      return undefined;
    }

    const {
      content,
      encoding,
      fileName,
      mapping,
      primaryTeamName,
      resolutions,
      sheetName,
      sourceName,
    } = payload as Record<string, unknown>;
    const parsedResolutions = readResolutions(resolutions);

    if (
      typeof content !== 'string' ||
      !content.trim() ||
      (encoding !== 'base64' && encoding !== 'text') ||
      typeof fileName !== 'string' ||
      !fileName.trim() ||
      (sheetName !== undefined && (typeof sheetName !== 'string' || !sheetName.trim())) ||
      typeof mapping !== 'object' ||
      mapping === null ||
      typeof primaryTeamName !== 'string' ||
      !primaryTeamName.trim() ||
      parsedResolutions === undefined ||
      typeof sourceName !== 'string' ||
      !sourceName.trim()
    ) {
      return undefined;
    }

    return {
      mapping: mapping as GameImportMapping,
      primaryTeamName: primaryTeamName.trim(),
      resolutions: parsedResolutions,
      sourceName: sourceName.trim(),
      upload: {
        content,
        encoding: encoding as GameImportFileEncoding,
        fileName: fileName.trim(),
        ...(typeof sheetName === 'string' ? { sheetName: sheetName.trim() } : {}),
      },
    };
  } catch {
    return undefined;
  }
};

export const POST = async ({ request }) => {
  const input = await readRequest(request);

  if (!input) {
    return json({ error: 'invalid_game_import' }, { status: 400 });
  }

  let preview;

  try {
    preview = previewGameImport(readGameImportFile(input.upload), input.mapping);
  } catch (error) {
    return json(
      { error: error instanceof Error ? error.message : 'The import file could not be read.' },
      { status: 400 },
    );
  }

  if (preview.issues.length > 0) {
    return json({ error: 'invalid_import_rows', preview }, { status: 400 });
  }

  if (preview.records.length === 0) {
    return json({ error: 'empty_game_import' }, { status: 400 });
  }

  const gameRepository = currentGameRepository();
  const analysis = await findImportConflicts(gameRepository, preview.records);
  const resolvedRows = new Set(input.resolutions.map((resolution) => resolution.sourceRow));
  const unresolvedConflicts = analysis.conflicts.filter(
    (conflict) => !resolvedRows.has(conflict.sourceRow),
  );

  if (unresolvedConflicts.length > 0) {
    return json({ conflicts: unresolvedConflicts, error: 'import_conflicts' }, { status: 409 });
  }

  try {
    const result = await withCurrentImportTransaction(async ({ audit, gameImports, games }) => {
      const importedResult = await importGames(games, gameImports, {
        atomic: true,
        importedAt: new Date(),
        primaryTeamName: input.primaryTeamName,
        records: preview.records,
        resolutions: input.resolutions,
        sourceName: input.sourceName,
      });

      await audit.record({
        action: 'games_imported',
        entityId: input.sourceName,
        entityType: 'game_import',
        metadata: {
          conflicts: importedResult.conflicts.length,
          duplicates: importedResult.duplicates.length,
          failed: importedResult.failed.length,
          imported: importedResult.imported.length,
          sourceName: input.sourceName,
        },
      });

      return importedResult;
    });

    return json(result);
  } catch (error) {
    if (error instanceof Error && error.message.includes('does not exist')) {
      return json({ error: 'game_import_failed' }, { status: 400 });
    }

    throw error;
  }
};
