import type { LocationRepository } from '../locations/location-repository';
import type { ImportedLocation } from './location-import';

export interface UpdatedLocationResult extends ImportedLocation {
  previousTravelMinutes: number;
}

export interface UnchangedLocationResult {
  sourceRow: number;
  travelMinutes: number;
}

export interface FailedLocationResult {
  message: string;
  sourceRow: number;
}

export interface LocationImportResult {
  failed: FailedLocationResult[];
  imported: ImportedLocation[];
  unchanged: UnchangedLocationResult[];
  updated: UpdatedLocationResult[];
}

export const importLocations = async (
  locations: LocationRepository,
  records: ImportedLocation[],
): Promise<LocationImportResult> => {
  const imported: ImportedLocation[] = [];
  const updated: UpdatedLocationResult[] = [];
  const unchanged: UnchangedLocationResult[] = [];
  const failed: FailedLocationResult[] = [];

  for (const record of records) {
    try {
      const existing = await locations.findByName(record.name);

      if (!existing) {
        await locations.save({
          ...(record.address ? { address: record.address } : {}),
          name: record.name,
          travelMinutes: record.travelMinutes,
        });
        imported.push(record);
        continue;
      }

      const nextAddress = record.address ?? existing.address;

      if (existing.travelMinutes === record.travelMinutes && existing.address === nextAddress) {
        unchanged.push({ sourceRow: record.sourceRow, travelMinutes: existing.travelMinutes });
        continue;
      }

      await locations.updateName(record.name, {
        ...(nextAddress ? { address: nextAddress } : {}),
        name: record.name,
        travelMinutes: record.travelMinutes,
      });
      updated.push({ ...record, previousTravelMinutes: existing.travelMinutes });
    } catch (error) {
      failed.push({
        message: error instanceof Error ? error.message : 'The location could not be imported.',
        sourceRow: record.sourceRow,
      });
    }
  }

  return { failed, imported, unchanged, updated };
};
