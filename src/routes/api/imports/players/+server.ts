import { json } from '@sveltejs/kit';

import {
  playerImportFields,
  previewPlayerImport,
  type PlayerImportMapping,
} from '$lib/application/imports/player-import';
import { importPlayers } from '$lib/application/imports/import-players';
import {
  readImportFile,
  type ImportFileEncoding,
  type ImportUpload,
} from '$lib/server/imports/spreadsheet-upload';
import {
  currentCoordinatorSettingsRepository,
  withCurrentImportTransaction,
} from '$lib/server/composition-root';

interface ImportRequest {
  mapping: PlayerImportMapping;
  sourceName: string;
  upload: ImportUpload;
}

const readRequest = async (request: Request): Promise<ImportRequest | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload !== 'object' || payload === null) {
      return undefined;
    }

    const { content, encoding, fileName, mapping, sheetName, sourceName } = payload as Record<
      string,
      unknown
    >;

    if (
      typeof content !== 'string' ||
      !content.trim() ||
      (encoding !== 'base64' && encoding !== 'text') ||
      typeof fileName !== 'string' ||
      !fileName.trim() ||
      (sheetName !== undefined && (typeof sheetName !== 'string' || !sheetName.trim())) ||
      typeof mapping !== 'object' ||
      mapping === null ||
      typeof sourceName !== 'string' ||
      !sourceName.trim()
    ) {
      return undefined;
    }

    const validatedMapping: PlayerImportMapping = {};
    const values = mapping as Record<string, unknown>;

    for (const field of playerImportFields) {
      const value = values[field];

      if (value !== undefined && typeof value !== 'string') {
        return undefined;
      }

      if (typeof value === 'string' && value.trim()) {
        validatedMapping[field] = value.trim();
      }
    }

    return {
      mapping: validatedMapping,
      sourceName: sourceName.trim(),
      upload: {
        content,
        encoding: encoding as ImportFileEncoding,
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
    return json({ error: 'invalid_player_import' }, { status: 400 });
  }

  const settings = await currentCoordinatorSettingsRepository().get();

  if (!settings) {
    return json({ error: 'coordinator_settings_not_configured' }, { status: 400 });
  }

  let preview;

  try {
    preview = previewPlayerImport(readImportFile(input.upload), input.mapping);
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
    return json({ error: 'empty_player_import' }, { status: 400 });
  }

  const result = await withCurrentImportTransaction(async ({ audit, memberships, players }) => {
    const importedResult = await importPlayers(players, memberships, {
      context: {
        primaryTeamName: settings.primaryTeamName,
        seasonStartingYear: settings.seasonStartingYear,
      },
      records: preview.records,
    });

    await audit.record({
      action: 'players_imported',
      entityId: input.sourceName,
      entityType: 'player_import',
      metadata: {
        failed: importedResult.failed.length,
        imported: importedResult.imported.length,
        sourceName: input.sourceName,
      },
    });

    return importedResult;
  });

  return json(result);
};
