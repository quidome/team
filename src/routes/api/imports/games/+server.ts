import { json } from '@sveltejs/kit';

import { previewGameImport, type GameImportMapping } from '$lib/application/imports/game-import';
import { importGames } from '$lib/application/imports/import-games';
import {
  readGameImportFile,
  type GameImportFileEncoding,
  type GameImportUpload,
} from '$lib/server/imports/game-import-upload';
import { currentGameImportRepository, currentGameRepository } from '$lib/server/composition-root';

interface ImportRequest {
  mapping: GameImportMapping;
  primaryTeamName: string;
  sourceName: string;
  upload: GameImportUpload;
}

const readRequest = async (request: Request): Promise<ImportRequest | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload !== 'object' || payload === null) {
      return undefined;
    }

    const { content, encoding, fileName, mapping, primaryTeamName, sourceName } = payload as Record<
      string,
      unknown
    >;

    if (
      typeof content !== 'string' ||
      !content.trim() ||
      (encoding !== 'base64' && encoding !== 'text') ||
      typeof fileName !== 'string' ||
      !fileName.trim() ||
      typeof mapping !== 'object' ||
      mapping === null ||
      typeof primaryTeamName !== 'string' ||
      !primaryTeamName.trim() ||
      typeof sourceName !== 'string' ||
      !sourceName.trim()
    ) {
      return undefined;
    }

    return {
      mapping: mapping as GameImportMapping,
      primaryTeamName: primaryTeamName.trim(),
      sourceName: sourceName.trim(),
      upload: {
        content,
        encoding: encoding as GameImportFileEncoding,
        fileName: fileName.trim(),
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

  return json(
    await importGames(currentGameRepository(), currentGameImportRepository(), {
      importedAt: new Date(),
      primaryTeamName: input.primaryTeamName,
      records: preview.records,
      sourceName: input.sourceName,
    }),
  );
};
