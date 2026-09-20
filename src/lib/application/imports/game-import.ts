export const gameImportFields = [
  'awayTeamName',
  'date',
  'homeTeamName',
  'locationName',
  'startTime',
  'travelMinutes',
  'arrivalBufferMinutes',
] as const;

export type GameImportField = (typeof gameImportFields)[number];

export type GameImportMapping = Partial<Record<GameImportField, string>>;

export interface ImportedGame {
  arrivalBufferMinutes: number;
  awayTeamName: string;
  date: string;
  homeTeamName: string;
  locationName: string;
  sourceRow: number;
  startTime: string;
  travelMinutes: number;
}

export interface ImportIssue {
  field?: GameImportField;
  message: string;
  row: number;
}

export interface GameImportPreview {
  headers: string[];
  issues: ImportIssue[];
  records: ImportedGame[];
  validRowCount: number;
}

const requiredFields = new Set<GameImportField>([
  'awayTeamName',
  'date',
  'homeTeamName',
  'locationName',
  'startTime',
]);

const parseCsvRows = (content: string): string[][] => {
  const rows: string[][] = [];
  let current = '';
  let row: string[] = [];
  let quoted = false;

  const pushCell = () => {
    row.push(current);
    current = '';
  };
  const pushRow = () => {
    if (row.length > 1 || row[0]?.trim()) {
      rows.push(row);
    }
    row = [];
  };

  for (let index = 0; index < content.length; index += 1) {
    const character = content[index];
    const next = content[index + 1];

    if (character === '"') {
      if (quoted && next === '"') {
        current += '"';
        index += 1;
      } else {
        quoted = !quoted;
      }
    } else if (character === ',' && !quoted) {
      pushCell();
    } else if ((character === '\n' || character === '\r') && !quoted) {
      if (character === '\r' && next === '\n') {
        index += 1;
      }
      pushCell();
      pushRow();
    } else {
      current += character;
    }
  }

  if (current || row.length > 0) {
    pushCell();
    pushRow();
  }

  return rows;
};

const isCalendarDate = (value: string) => {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    return false;
  }

  const date = new Date(`${value}T00:00:00.000Z`);

  return Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === value;
};

const isTime = (value: string) => {
  if (!/^\d{2}:\d{2}$/.test(value)) {
    return false;
  }

  const [hours, minutes] = value.split(':').map(Number);

  return hours >= 0 && hours < 24 && minutes >= 0 && minutes < 60;
};

const readValue = (
  row: Record<string, string>,
  mapping: GameImportMapping,
  field: GameImportField,
): string => (mapping[field] ? (row[mapping[field]]?.trim() ?? '') : '');

const readOptionalNumber = (
  row: Record<string, string>,
  mapping: GameImportMapping,
  field: 'arrivalBufferMinutes' | 'travelMinutes',
  fallback: number,
  rowNumber: number,
  issues: ImportIssue[],
): number => {
  const value = readValue(row, mapping, field);

  if (!value) {
    return fallback;
  }

  const number = Number(value);

  if (!Number.isInteger(number) || number < 0) {
    issues.push({ field, message: 'Must be a non-negative whole number.', row: rowNumber });
    return fallback;
  }

  return number;
};

export const previewGameImport = (
  content: string,
  mapping: GameImportMapping,
): GameImportPreview => {
  const rows = parseCsvRows(content);
  const headers = (rows.shift() ?? []).map((header, index) =>
    (index === 0 ? header.replace(/^\uFEFF/, '') : header).trim(),
  );
  const issues: ImportIssue[] = [];
  const records: ImportedGame[] = [];

  for (const field of gameImportFields) {
    if (requiredFields.has(field) && !mapping[field]) {
      issues.push({ field, message: 'Map a source column for this required field.', row: 1 });
    } else if (mapping[field] && !headers.includes(mapping[field])) {
      issues.push({ field, message: 'The mapped source column does not exist.', row: 1 });
    }
  }

  rows.forEach((values, index) => {
    const rowNumber = index + 2;
    const row = Object.fromEntries(headers.map((header, column) => [header, values[column] ?? '']));
    const date = readValue(row, mapping, 'date');
    const startTime = readValue(row, mapping, 'startTime');
    const homeTeamName = readValue(row, mapping, 'homeTeamName');
    const awayTeamName = readValue(row, mapping, 'awayTeamName');
    const locationName = readValue(row, mapping, 'locationName');
    const rowIssues: ImportIssue[] = [];

    if (date && !isCalendarDate(date)) {
      rowIssues.push({
        field: 'date',
        message: 'Use a valid date in YYYY-MM-DD format.',
        row: rowNumber,
      });
    }
    if (startTime && !isTime(startTime)) {
      rowIssues.push({
        field: 'startTime',
        message: 'Use a valid time in HH:MM format.',
        row: rowNumber,
      });
    }
    if (homeTeamName && awayTeamName && homeTeamName === awayTeamName) {
      rowIssues.push({ message: 'Home and away teams must be different.', row: rowNumber });
    }
    for (const [field, value] of [
      ['homeTeamName', homeTeamName],
      ['awayTeamName', awayTeamName],
      ['date', date],
      ['startTime', startTime],
      ['locationName', locationName],
    ] as const) {
      if (!value) {
        rowIssues.push({ field, message: 'A value is required.', row: rowNumber });
      }
    }

    const travelMinutes = readOptionalNumber(
      row,
      mapping,
      'travelMinutes',
      0,
      rowNumber,
      rowIssues,
    );
    const arrivalBufferMinutes = readOptionalNumber(
      row,
      mapping,
      'arrivalBufferMinutes',
      30,
      rowNumber,
      rowIssues,
    );

    issues.push(...rowIssues);
    if (rowIssues.length === 0) {
      records.push({
        arrivalBufferMinutes,
        awayTeamName,
        date,
        homeTeamName,
        locationName,
        sourceRow: rowNumber,
        startTime,
        travelMinutes,
      });
    }
  });

  return { headers, issues, records, validRowCount: records.length };
};
