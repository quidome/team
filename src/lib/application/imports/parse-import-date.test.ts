import { describe, expect, it } from 'vitest';

import { parseImportDate } from './parse-import-date';

describe('parseImportDate', () => {
  it('round-trips a valid ISO date', () => {
    expect(parseImportDate('2026-09-26')).toBe('2026-09-26');
  });

  it.each([
    ['15-08-2026', '2026-08-15'],
    ['15/08/2026', '2026-08-15'],
    ['3-10-2026', '2026-10-03'],
  ])('parses the day-first date %s as %s', (value, expected) => {
    expect(parseImportDate(value)).toBe(expected);
  });

  it('rejects a day-first date with an invalid day for its month', () => {
    expect(parseImportDate('31/4/2026')).toBeUndefined();
  });

  it('rejects an invalid ISO date', () => {
    expect(parseImportDate('2026-13-01')).toBeUndefined();
  });

  it('rejects an unrecognizable string', () => {
    expect(parseImportDate('not a date')).toBeUndefined();
  });
});
