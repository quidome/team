import { json } from '@sveltejs/kit';

import { previewGameImport, type GameImportMapping } from '$lib/application/imports/game-import';
import { importGames } from '$lib/application/imports/import-games';
import { currentGameImportRepository, currentGameRepository } from '$lib/server/composition-root';

interface ImportRequest {
  content: string;
  mapping: GameImportMapping;
  primaryTeamName: string;
  sourceName: string;
}

const readRequest = async (request: Request): Promise<ImportRequest | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload !== 'object' || payload === null) {
      return undefined;
    }

    const { content, mapping, primaryTeamName, sourceName } = payload as Record<string, unknown>;

    if (
      typeof content !== 'string' ||
      !content.trim() ||
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
      content,
      mapping: mapping as GameImportMapping,
      primaryTeamName: primaryTeamName.trim(),
      sourceName: sourceName.trim(),
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

  const preview = previewGameImport(input.content, input.mapping);

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
