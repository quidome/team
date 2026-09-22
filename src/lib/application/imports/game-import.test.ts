import { describe, expect, it } from 'vitest';

import { maxImportRows } from './import-limits';
import { previewGameImport } from './game-import';

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
      ...Array.from({ length: maxImportRows + 1 }, () => 'U16-1,U18-1,2026-08-15,14:30,Home'),
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

  it('accepts a day-first date and normalizes it to ISO', () => {
    const preview = previewGameImport(
      'home,away,date,time,location\nU16-1,U18-1,15-08-2026,14:30,Home court',
      {
        awayTeamName: 'away',
        date: 'date',
        homeTeamName: 'home',
        locationName: 'location',
        startTime: 'time',
      },
    );

    expect(preview.issues).toEqual([]);
    expect(preview.records).toEqual([expect.objectContaining({ date: '2026-08-15' })]);
  });

  it('rejects an invalid day-first date', () => {
    const preview = previewGameImport(
      'home,away,date,time,location\nU16-1,U18-1,31-04-2026,14:30,Home court',
      {
        awayTeamName: 'away',
        date: 'date',
        homeTeamName: 'home',
        locationName: 'location',
        startTime: 'time',
      },
    );

    expect(preview.validRowCount).toBe(0);
    expect(preview.issues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ field: 'date', message: expect.stringContaining('valid date') }),
      ]),
    );
  });

  it('reads jury and referee slot counts when mapped and present', () => {
    const preview = previewGameImport(
      'home,away,date,time,location,jury,referee\nU16-1,U18-1,2026-08-15,14:30,Home court,2,1',
      {
        awayTeamName: 'away',
        date: 'date',
        homeTeamName: 'home',
        jurySlots: 'jury',
        locationName: 'location',
        refereeSlots: 'referee',
        startTime: 'time',
      },
    );

    expect(preview.issues).toEqual([]);
    expect(preview.records).toEqual([expect.objectContaining({ jurySlots: 2, refereeSlots: 1 })]);
  });

  it('leaves jury and referee slots unset rather than defaulting to zero when blank', () => {
    const preview = previewGameImport(
      'home,away,date,time,location,jury,referee\nU16-1,U18-1,2026-08-15,14:30,Home court,,',
      {
        awayTeamName: 'away',
        date: 'date',
        homeTeamName: 'home',
        jurySlots: 'jury',
        locationName: 'location',
        refereeSlots: 'referee',
        startTime: 'time',
      },
    );

    expect(preview.issues).toEqual([]);
    const [record] = preview.records;
    expect(record?.jurySlots).toBeUndefined();
    expect(record?.refereeSlots).toBeUndefined();
  });

  it('rejects a jury or referee slot count outside 0-2', () => {
    const preview = previewGameImport(
      'home,away,date,time,location,jury\nU16-1,U18-1,2026-08-15,14:30,Home court,3',
      {
        awayTeamName: 'away',
        date: 'date',
        homeTeamName: 'home',
        jurySlots: 'jury',
        locationName: 'location',
        startTime: 'time',
      },
    );

    expect(preview.validRowCount).toBe(0);
    expect(preview.issues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          field: 'jurySlots',
          message: 'Must be a whole number from 0 to 2.',
        }),
      ]),
    );
  });
});
