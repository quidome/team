import { parseCsvRows } from './parse-import-csv';

export const locationImportFields = ['name', 'travelMinutes'] as const;

export type LocationImportField = (typeof locationImportFields)[number];

export type LocationImportMapping = Partial<Record<LocationImportField, string>>;

export interface ImportedLocation {
  name: string;
  sourceRow: number;
  travelMinutes: number;
}

export interface ImportIssue {
  field?: LocationImportField;
  message: string;
  row: number;
}

export interface LocationImportPreview {
  headers: string[];
  issues: ImportIssue[];
  records: ImportedLocation[];
  validRowCount: number;
}

const requiredFields = new Set<LocationImportField>(['name']);

const readValue = (
  row: Record<string, string>,
  mapping: LocationImportMapping,
  field: LocationImportField,
): string => (mapping[field] ? (row[mapping[field]]?.trim() ?? '') : '');

export const previewLocationImport = (
  content: string,
  mapping: LocationImportMapping,
): LocationImportPreview => {
  const rows = parseCsvRows(content);
  const headers = (rows.shift() ?? []).map((header, index) =>
    (index === 0 ? header.replace(/^\uFEFF/, '') : header).trim(),
  );
  const issues: ImportIssue[] = [];
  const records: ImportedLocation[] = [];

  for (const field of locationImportFields) {
    if (requiredFields.has(field) && !mapping[field]) {
      issues.push({ field, message: 'Map a source column for this required field.', row: 1 });
    } else if (mapping[field] && !headers.includes(mapping[field])) {
      issues.push({ field, message: 'The mapped source column does not exist.', row: 1 });
    }
  }

  rows.forEach((values, index) => {
    const rowNumber = index + 2;
    const row = Object.fromEntries(headers.map((header, column) => [header, values[column] ?? '']));
    const name = readValue(row, mapping, 'name');
    const travelMinutesRaw = readValue(row, mapping, 'travelMinutes');
    const rowIssues: ImportIssue[] = [];

    if (!name) {
      rowIssues.push({ field: 'name', message: 'A value is required.', row: rowNumber });
    }

    let travelMinutes = 0;

    if (travelMinutesRaw) {
      const number = Number(travelMinutesRaw);

      if (!Number.isInteger(number) || number < 0) {
        rowIssues.push({
          field: 'travelMinutes',
          message: 'Must be a non-negative whole number.',
          row: rowNumber,
        });
      } else {
        travelMinutes = number;
      }
    }

    issues.push(...rowIssues);
    if (rowIssues.length === 0) {
      records.push({ name, sourceRow: rowNumber, travelMinutes });
    }
  });

  return { headers, issues, records, validRowCount: records.length };
};
