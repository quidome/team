import { describe, expect, it } from 'vitest';
import * as XLSX from 'xlsx';

import { maxImportColumns, maxImportRows } from '$lib/application/imports/import-limits';

import { listImportWorksheets, readImportFile } from './spreadsheet-upload';

describe('readImportFile', () => {
  it('converts the first XLSX worksheet to the CSV import format', () => {
    const workbook = XLSX.utils.book_new();
    const sheet = XLSX.utils.aoa_to_sheet([
      ['home', 'away', 'date'],
      ['U16-1', 'U18-1', '2026-08-15'],
    ]);
    XLSX.utils.book_append_sheet(workbook, sheet, 'Games');

    const content = XLSX.write(workbook, { bookType: 'xlsx', type: 'base64' });

    expect(readImportFile({ content, encoding: 'base64', fileName: 'schedule.xlsx' })).toContain(
      'U16-1,U18-1,2026-08-15',
    );
  });

  it('lists worksheets and reads a selected worksheet', () => {
    const workbook = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(
      workbook,
      XLSX.utils.aoa_to_sheet([['home'], ['wrong-sheet']]),
      'Notes',
    );
    XLSX.utils.book_append_sheet(
      workbook,
      XLSX.utils.aoa_to_sheet([
        ['home', 'away', 'date'],
        ['U16-1', 'U18-1', '2026-08-15'],
      ]),
      'Games',
    );
    const content = XLSX.write(workbook, { bookType: 'xlsx', type: 'base64' });

    expect(
      listImportWorksheets({ content, encoding: 'base64', fileName: 'schedule.xlsx' }),
    ).toEqual(['Notes', 'Games']);
    expect(
      readImportFile({
        content,
        encoding: 'base64',
        fileName: 'schedule.xlsx',
        sheetName: 'Games',
      }),
    ).toContain('U16-1,U18-1,2026-08-15');
    expect(() =>
      readImportFile({
        content,
        encoding: 'base64',
        fileName: 'schedule.xlsx',
        sheetName: 'Missing',
      }),
    ).toThrow('worksheet named "Missing"');
  });

  it('rejects unsupported file extensions', () => {
    expect(() =>
      readImportFile({ content: 'anything', encoding: 'text', fileName: 'schedule.pdf' }),
    ).toThrow('Supported import files');
  });

  it('rejects spreadsheets with too many data rows', () => {
    const workbook = XLSX.utils.book_new();
    const sheet = XLSX.utils.aoa_to_sheet([
      ['home'],
      ...Array.from({ length: maxImportRows + 1 }, () => ['U16-1']),
    ]);
    XLSX.utils.book_append_sheet(workbook, sheet, 'Games');

    const content = XLSX.write(workbook, { bookType: 'xlsx', type: 'base64' });

    expect(() =>
      readImportFile({ content, encoding: 'base64', fileName: 'schedule.xlsx' }),
    ).toThrow('10,000 data rows');
  });

  it('rejects spreadsheets with too many columns', () => {
    const workbook = XLSX.utils.book_new();
    const sheet = XLSX.utils.aoa_to_sheet([
      Array.from({ length: maxImportColumns + 1 }, (_, index) => `column-${index}`),
    ]);
    XLSX.utils.book_append_sheet(workbook, sheet, 'Games');

    const content = XLSX.write(workbook, { bookType: 'xlsx', type: 'base64' });

    expect(() =>
      readImportFile({ content, encoding: 'base64', fileName: 'schedule.xlsx' }),
    ).toThrow('100 columns');
  });

  it('rejects import files larger than 10 MiB', () => {
    expect(() =>
      readImportFile({
        content: 'x'.repeat(10 * 1024 * 1024 + 1),
        encoding: 'text',
        fileName: 'schedule.csv',
      }),
    ).toThrow('must not exceed 10 MiB');
  });
});
