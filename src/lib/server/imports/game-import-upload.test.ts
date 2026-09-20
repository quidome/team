import { describe, expect, it } from 'vitest';
import * as XLSX from 'xlsx';

import { maxGameImportColumns, maxGameImportRows } from '$lib/application/imports/game-import';

import { readGameImportFile } from './game-import-upload';

describe('readGameImportFile', () => {
  it('converts the first XLSX worksheet to the CSV import format', () => {
    const workbook = XLSX.utils.book_new();
    const sheet = XLSX.utils.aoa_to_sheet([
      ['home', 'away', 'date'],
      ['U16-1', 'U18-1', '2026-08-15'],
    ]);
    XLSX.utils.book_append_sheet(workbook, sheet, 'Games');

    const content = XLSX.write(workbook, { bookType: 'xlsx', type: 'base64' });

    expect(
      readGameImportFile({ content, encoding: 'base64', fileName: 'schedule.xlsx' }),
    ).toContain('U16-1,U18-1,2026-08-15');
  });

  it('rejects unsupported file extensions', () => {
    expect(() =>
      readGameImportFile({ content: 'anything', encoding: 'text', fileName: 'schedule.pdf' }),
    ).toThrow('Supported schedule files');
  });

  it('rejects spreadsheets with too many data rows', () => {
    const workbook = XLSX.utils.book_new();
    const sheet = XLSX.utils.aoa_to_sheet([
      ['home'],
      ...Array.from({ length: maxGameImportRows + 1 }, () => ['U16-1']),
    ]);
    XLSX.utils.book_append_sheet(workbook, sheet, 'Games');

    const content = XLSX.write(workbook, { bookType: 'xlsx', type: 'base64' });

    expect(() =>
      readGameImportFile({ content, encoding: 'base64', fileName: 'schedule.xlsx' }),
    ).toThrow('10,000 data rows');
  });

  it('rejects spreadsheets with too many columns', () => {
    const workbook = XLSX.utils.book_new();
    const sheet = XLSX.utils.aoa_to_sheet([
      Array.from({ length: maxGameImportColumns + 1 }, (_, index) => `column-${index}`),
    ]);
    XLSX.utils.book_append_sheet(workbook, sheet, 'Games');

    const content = XLSX.write(workbook, { bookType: 'xlsx', type: 'base64' });

    expect(() =>
      readGameImportFile({ content, encoding: 'base64', fileName: 'schedule.xlsx' }),
    ).toThrow('100 columns');
  });

  it('rejects schedule files larger than 10 MiB', () => {
    expect(() =>
      readGameImportFile({
        content: 'x'.repeat(10 * 1024 * 1024 + 1),
        encoding: 'text',
        fileName: 'schedule.csv',
      }),
    ).toThrow('must not exceed 10 MiB');
  });
});
