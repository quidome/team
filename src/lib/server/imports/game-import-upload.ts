import * as XLSX from 'xlsx';

export type GameImportFileEncoding = 'base64' | 'text';

export interface GameImportUpload {
  content: string;
  encoding: GameImportFileEncoding;
  fileName: string;
}

const supportedExtensions = new Set(['.csv', '.ods', '.xls', '.xlsx']);

const extensionOf = (fileName: string) => {
  const dot = fileName.lastIndexOf('.');

  return dot === -1 ? '' : fileName.slice(dot).toLowerCase();
};

export const readGameImportFile = (upload: GameImportUpload): string => {
  const extension = extensionOf(upload.fileName);

  if (!supportedExtensions.has(extension)) {
    throw new Error('Supported schedule files are CSV, XLS, XLSX, and ODS.');
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
    workbook = XLSX.read(Buffer.from(upload.content, 'base64'), { type: 'buffer' });
  } catch {
    throw new Error('The spreadsheet could not be read.');
  }

  const firstSheetName = workbook.SheetNames[0];
  const firstSheet = firstSheetName ? workbook.Sheets[firstSheetName] : undefined;

  if (!firstSheet) {
    throw new Error('The spreadsheet does not contain a worksheet.');
  }

  const csv = XLSX.utils.sheet_to_csv(firstSheet);

  if (!csv.trim()) {
    throw new Error('The spreadsheet worksheet is empty.');
  }

  return csv;
};
