import { parseImportDate } from './parse-import-date';
import { parseCsvRows } from './parse-import-csv';

export const playerImportFields = [
  'firstName',
  'lastName',
  'birthDate',
  'associationId',
  'jerseyNumber',
  'participationType',
] as const;

export type PlayerImportField = (typeof playerImportFields)[number];

export type PlayerImportMapping = Partial<Record<PlayerImportField, string>>;

export type PlayerImportParticipationType = 'trains_and_plays' | 'trains_only';

export interface ImportedPlayer {
  associationId?: string;
  birthDate?: string;
  firstName: string;
  jerseyNumber?: number;
  lastName?: string;
  participationType: PlayerImportParticipationType;
  sourceRow: number;
}

export interface ImportIssue {
  field?: PlayerImportField;
  message: string;
  row: number;
}

export interface PlayerImportPreview {
  headers: string[];
  issues: ImportIssue[];
  records: ImportedPlayer[];
  validRowCount: number;
}

const requiredFields = new Set<PlayerImportField>(['firstName']);

const participationTypeAliases: Record<string, PlayerImportParticipationType> = {
  game: 'trains_and_plays',
  games: 'trains_and_plays',
  'trains and plays': 'trains_and_plays',
  trains_and_plays: 'trains_and_plays',
  'trains only': 'trains_only',
  trains_only: 'trains_only',
  training: 'trains_only',
  trainings: 'trains_only',
};

const readValue = (
  row: Record<string, string>,
  mapping: PlayerImportMapping,
  field: PlayerImportField,
): string => (mapping[field] ? (row[mapping[field]]?.trim() ?? '') : '');

export const previewPlayerImport = (
  content: string,
  mapping: PlayerImportMapping,
): PlayerImportPreview => {
  const rows = parseCsvRows(content);
  const headers = (rows.shift() ?? []).map((header, index) =>
    (index === 0 ? header.replace(/^\uFEFF/, '') : header).trim(),
  );
  const issues: ImportIssue[] = [];
  const records: ImportedPlayer[] = [];

  for (const field of playerImportFields) {
    if (requiredFields.has(field) && !mapping[field]) {
      issues.push({ field, message: 'Map a source column for this required field.', row: 1 });
    } else if (mapping[field] && !headers.includes(mapping[field])) {
      issues.push({ field, message: 'The mapped source column does not exist.', row: 1 });
    }
  }

  rows.forEach((values, index) => {
    const rowNumber = index + 2;
    const row = Object.fromEntries(headers.map((header, column) => [header, values[column] ?? '']));
    const firstName = readValue(row, mapping, 'firstName');
    const lastName = readValue(row, mapping, 'lastName');
    const birthDateRaw = readValue(row, mapping, 'birthDate');
    const associationId = readValue(row, mapping, 'associationId');
    const jerseyNumberRaw = readValue(row, mapping, 'jerseyNumber');
    const participationTypeRaw = readValue(row, mapping, 'participationType');
    const rowIssues: ImportIssue[] = [];

    if (!firstName) {
      rowIssues.push({ field: 'firstName', message: 'A value is required.', row: rowNumber });
    }

    let birthDate: string | undefined;

    if (birthDateRaw) {
      const parsedDate = parseImportDate(birthDateRaw);

      if (!parsedDate) {
        rowIssues.push({
          field: 'birthDate',
          message: 'Use a valid date (YYYY-MM-DD or DD-MM-YYYY).',
          row: rowNumber,
        });
      } else {
        birthDate = parsedDate;
      }
    }

    let jerseyNumber: number | undefined;

    if (jerseyNumberRaw) {
      const number = Number(jerseyNumberRaw);

      if (!Number.isInteger(number) || number <= 0) {
        rowIssues.push({
          field: 'jerseyNumber',
          message: 'Must be a positive whole number.',
          row: rowNumber,
        });
      } else {
        jerseyNumber = number;
      }
    }

    let participationType: PlayerImportParticipationType = 'trains_and_plays';

    if (participationTypeRaw) {
      const normalized = participationTypeAliases[participationTypeRaw.toLowerCase()];

      if (!normalized) {
        rowIssues.push({
          field: 'participationType',
          message: 'Use "trains and plays" or "trains only".',
          row: rowNumber,
        });
      } else {
        participationType = normalized;
      }
    }

    issues.push(...rowIssues);
    if (rowIssues.length === 0) {
      records.push({
        ...(associationId ? { associationId } : {}),
        ...(birthDate ? { birthDate } : {}),
        firstName,
        ...(jerseyNumber !== undefined ? { jerseyNumber } : {}),
        ...(lastName ? { lastName } : {}),
        participationType,
        sourceRow: rowNumber,
      });
    }
  });

  return { headers, issues, records, validRowCount: records.length };
};
