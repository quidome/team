import * as XLSX from 'xlsx';

import {
  maxImportBytes,
  maxImportColumns,
  maxImportRows,
} from '$lib/application/imports/import-limits';

export type ImportFileEncoding = 'base64' | 'text';

export interface ImportUpload {
  content: string;
  encoding: ImportFileEncoding;
  fileName: string;
  sheetName?: string;
}

const supportedExtensions = new Set(['.csv', '.ods', '.xls', '.xlsx']);

const extensionOf = (fileName: string) => {
  const dot = fileName.lastIndexOf('.');

  return dot === -1 ? '' : fileName.slice(dot).toLowerCase();
};

export const listImportWorksheets = (upload: ImportUpload): string[] => {
  const extension = extensionOf(upload.fileName);

  if (!supportedExtensions.has(extension)) {
    throw new Error('Supported import files are CSV, XLS, XLSX, and ODS.');
  }

  if (extension === '.csv') {
    return [];
  }

  if (upload.encoding !== 'base64') {
    throw new Error('Spreadsheet files must be uploaded as binary data.');
  }

  const byteLength = Buffer.from(upload.content, 'base64').byteLength;

  if (byteLength > maxImportBytes) {
    throw new Error('Import files must not exceed 10 MiB.');
  }

  try {
    return XLSX.read(Buffer.from(upload.content, 'base64'), {
      sheetRows: maxImportRows + 2,
      type: 'buffer',
    }).SheetNames;
  } catch {
    throw new Error('The spreadsheet could not be read.');
  }
};

export const readImportFile = (upload: ImportUpload): string => {
  const extension = extensionOf(upload.fileName);

  if (!supportedExtensions.has(extension)) {
    throw new Error('Supported import files are CSV, XLS, XLSX, and ODS.');
  }

  const byteLength =
    upload.encoding === 'base64'
      ? Buffer.from(upload.content, 'base64').byteLength
      : Buffer.byteLength(upload.content, 'utf8');

  if (byteLength > maxImportBytes) {
    throw new Error('Import files must not exceed 10 MiB.');
  }

  if (extension === '.csv') {
    if (upload.encoding !== 'text') {
      throw new Error('CSV files must be uploaded as text.');
    }

    return upload.content;
  }

  if (upload.encoding !== 'base64') {
    throw new Error('Spreadsheet files must be uploaded as binary data.');
  }

  let workbook: XLSX.WorkBook;

  try {
    workbook = XLSX.read(Buffer.from(upload.content, 'base64'), {
      sheetRows: maxImportRows + 2,
      type: 'buffer',
    });
  } catch {
    throw new Error('The spreadsheet could not be read.');
  }

  const requestedSheetName = upload.sheetName?.trim();
  const selectedSheetName = requestedSheetName || workbook.SheetNames[0];
  const selectedSheet = selectedSheetName ? workbook.Sheets[selectedSheetName] : undefined;

  if (requestedSheetName && !selectedSheet) {
    throw new Error(`The spreadsheet does not contain a worksheet named "${requestedSheetName}".`);
  }

  if (!selectedSheet) {
    throw new Error('The spreadsheet does not contain a worksheet.');
  }

  const range = selectedSheet['!ref'] ? XLSX.utils.decode_range(selectedSheet['!ref']) : undefined;

  if (range && range.e.r >= maxImportRows + 1) {
    throw new Error(
      `Import files must not contain more than ${maxImportRows.toLocaleString('en-US')} data rows.`,
    );
  }

  if (range && range.e.c >= maxImportColumns) {
    throw new Error(`Import files must not contain more than ${maxImportColumns} columns.`);
  }

  const csv = XLSX.utils.sheet_to_csv(selectedSheet);

  if (!csv.trim()) {
    throw new Error('The spreadsheet worksheet is empty.');
  }

  if (Buffer.byteLength(csv, 'utf8') > maxImportBytes) {
    throw new Error('The expanded spreadsheet must not exceed 10 MiB.');
  }

  return csv;
};
