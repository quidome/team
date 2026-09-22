import { json } from '@sveltejs/kit';

import {
  locationImportFields,
  previewLocationImport,
  type LocationImportMapping,
} from '$lib/application/imports/location-import';
import { importLocations } from '$lib/application/imports/import-locations';
import {
  readImportFile,
  type ImportFileEncoding,
  type ImportUpload,
} from '$lib/server/imports/spreadsheet-upload';
import { currentAuditRepository, currentLocationRepository } from '$lib/server/composition-root';

interface ImportRequest {
  mapping: LocationImportMapping;
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

    const validatedMapping: LocationImportMapping = {};
    const values = mapping as Record<string, unknown>;

    for (const field of locationImportFields) {
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
    return json({ error: 'invalid_location_import' }, { status: 400 });
  }

  let preview;

  try {
    preview = previewLocationImport(readImportFile(input.upload), input.mapping);
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
    return json({ error: 'empty_location_import' }, { status: 400 });
  }

  const result = await importLocations(currentLocationRepository(), preview.records);

  await currentAuditRepository().record({
    action: 'locations_imported',
    entityId: input.sourceName,
    entityType: 'location_import',
    metadata: {
      failed: result.failed.length,
      imported: result.imported.length,
      sourceName: input.sourceName,
      unchanged: result.unchanged.length,
      updated: result.updated.length,
    },
  });

  return json(result);
};
