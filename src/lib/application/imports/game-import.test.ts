import { describe, expect, it } from 'vitest';

import { maxGameImportRows, previewGameImport } from './game-import';

describe('game import preview', () => {
  it('parses quoted CSV values and validates mapped games', () => {
    const preview = previewGameImport(
      'home,away,date,time,location,travel\nU16-1,U18-1,2026-08-15,14:30,"Away, court",20',
      {
        awayTeamName: 'away',
        date: 'date',
        homeTeamName: 'home',
        locationName: 'location',
        startTime: 'time',
        travelMinutes: 'travel',
      },
    );

    expect(preview.headers).toEqual(['home', 'away', 'date', 'time', 'location', 'travel']);
    expect(preview.issues).toEqual([]);
    expect(preview.records).toEqual([
      expect.objectContaining({
        arrivalBufferMinutes: 30,
        locationName: 'Away, court',
        travelMinutes: 20,
      }),
    ]);
  });

  it('rejects CSV files with too many data rows', () => {
    const content = [
      'home,away,date,time,location',
      ...Array.from({ length: maxGameImportRows + 1 }, () => 'U16-1,U18-1,2026-08-15,14:30,Home'),
    ].join('\n');

    expect(() => previewGameImport(content, {})).toThrow('10,000 data rows');
  });

  it('reports unmapped required fields and invalid rows without importing them', () => {
    const preview = previewGameImport('home,away,date\nU16-1,U16-1,not-a-date', {
      awayTeamName: 'away',
      date: 'date',
      homeTeamName: 'home',
    });

    expect(preview.validRowCount).toBe(0);
    expect(preview.issues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ field: 'locationName', row: 1 }),
        expect.objectContaining({ field: 'startTime', row: 1 }),
        expect.objectContaining({ field: 'date', message: expect.stringContaining('valid date') }),
        expect.objectContaining({ message: 'Home and away teams must be different.' }),
      ]),
    );
  });
});
