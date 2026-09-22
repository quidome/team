import { maxImportColumns, maxImportRows } from './import-limits';

export const parseCsvRows = (content: string): string[][] => {
  const rows: string[][] = [];
  let current = '';
  let row: string[] = [];
  let quoted = false;

  const pushCell = () => {
    if (row.length >= maxImportColumns) {
      throw new Error(`Import files must not contain more than ${maxImportColumns} columns.`);
    }

    row.push(current);
    current = '';
  };
  const pushRow = () => {
    if (row.length > 1 || row[0]?.trim()) {
      rows.push(row);

      if (rows.length > maxImportRows + 1) {
        throw new Error(
          `Import files must not contain more than ${maxImportRows.toLocaleString('en-US')} data rows.`,
        );
      }
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
