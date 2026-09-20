import { json } from '@sveltejs/kit';

import {
  gameImportFields,
  previewGameImport,
  type GameImportMapping,
} from '$lib/application/imports/game-import';

const readRequest = async (
  request: Request,
): Promise<{ content: string; mapping: GameImportMapping } | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload !== 'object' || payload === null) {
      return undefined;
    }

    const { content, mapping } = payload as Record<string, unknown>;

    if (
      typeof content !== 'string' ||
      !content.trim() ||
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

    return { content, mapping: validatedMapping };
  } catch {
    return undefined;
  }
};

export const POST = async ({ request }) => {
  const input = await readRequest(request);

  if (!input) {
    return json({ error: 'invalid_import_preview' }, { status: 400 });
  }

  return json(previewGameImport(input.content, input.mapping));
};
