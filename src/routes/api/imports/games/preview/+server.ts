import { json } from '@sveltejs/kit';

import {
  gameImportFields,
  previewGameImport,
  type GameImportMapping,
} from '$lib/application/imports/game-import';
import { applyImportContext } from '$lib/application/imports/import-games';
import {
  listImportWorksheets,
  readImportFile,
  type ImportFileEncoding,
  type ImportUpload,
} from '$lib/server/imports/spreadsheet-upload';
import { buildGameImportContext } from '$lib/server/imports/game-import-context';

const readRequest = async (
  request: Request,
): Promise<{ mapping: GameImportMapping; upload: ImportUpload } | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload !== 'object' || payload === null) {
      return undefined;
    }

    const { content, encoding, fileName, mapping, sheetName } = payload as Record<string, unknown>;

    if (
      typeof content !== 'string' ||
      !content.trim() ||
      (encoding !== 'base64' && encoding !== 'text') ||
      typeof fileName !== 'string' ||
      !fileName.trim() ||
      (sheetName !== undefined && (typeof sheetName !== 'string' || !sheetName.trim())) ||
      typeof mapping !== 'object' ||
      mapping === null
    ) {
      return undefined;
    }

    const validatedMapping: GameImportMapping = {};
    const values = mapping as Record<string, unknown>;

    for (const field of gameImportFields) {
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
    return json({ error: 'invalid_import_preview' }, { status: 400 });
  }

  try {
    const worksheets = listImportWorksheets(input.upload);
    const context = await buildGameImportContext();
    const preview = previewGameImport(
      readImportFile(input.upload),
      input.mapping,
      context.knownLocationTravelMinutes,
    );

    if (Object.keys(input.mapping).length === 0) {
      return json({
        headers: preview.headers,
        issues: [],
        records: [],
        validRowCount: 0,
        worksheets,
      });
    }

    const contextualizedPreview = applyImportContext(preview, context);

    return json({ ...contextualizedPreview, worksheets });
  } catch (error) {
    return json(
      { error: error instanceof Error ? error.message : 'The import file could not be read.' },
      { status: 400 },
    );
  }
};
