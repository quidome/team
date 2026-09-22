import { json } from '@sveltejs/kit';

import {
  playerImportFields,
  previewPlayerImport,
  type PlayerImportMapping,
} from '$lib/application/imports/player-import';
import {
  listImportWorksheets,
  readImportFile,
  type ImportFileEncoding,
  type ImportUpload,
} from '$lib/server/imports/spreadsheet-upload';

const readRequest = async (
  request: Request,
): Promise<{ mapping: PlayerImportMapping; upload: ImportUpload } | undefined> => {
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
    const preview = previewPlayerImport(readImportFile(input.upload), input.mapping);

    if (Object.keys(input.mapping).length === 0) {
      return json({
        headers: preview.headers,
        issues: [],
        records: [],
        validRowCount: 0,
        worksheets,
      });
    }

    return json({ ...preview, worksheets });
  } catch (error) {
    return json(
      { error: error instanceof Error ? error.message : 'The import file could not be read.' },
      { status: 400 },
    );
  }
};
