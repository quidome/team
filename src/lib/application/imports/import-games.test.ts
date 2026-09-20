import { describe, expect, it } from 'vitest';

import { InMemoryGameImportRepository } from '../../adapters/in-memory-game-import-repository';
import { InMemoryGameRepository } from '../../adapters/in-memory-game-repository';
import { findImportConflicts, importGames } from './import-games';

describe('importGames', () => {
  it('imports all valid games, reuses fixtures, classifies primary-team games, and stores provenance', async () => {
    const games = new InMemoryGameRepository();
    const imports = new InMemoryGameImportRepository();
    const result = await importGames(games, imports, {
      importedAt: new Date('2026-08-01T10:00:00.000Z'),
      primaryTeamName: 'U16-1',
      records: [
        {
          arrivalBufferMinutes: 30,
          awayTeamName: 'U18-1',
          date: '2026-08-15',
          homeTeamName: 'U16-1',
          locationName: 'Away court',
          sourceRow: 2,
          startTime: '14:30',
          travelMinutes: 20,
        },
        {
          arrivalBufferMinutes: 30,
          awayTeamName: 'U18-1',
          date: '2026-08-22',
          homeTeamName: 'U16-1',
          locationName: 'Home court',
          sourceRow: 3,
          startTime: '11:00',
          travelMinutes: 0,
        },
      ],
      sourceName: 'schedule.csv',
    });

    expect(result.duplicates).toEqual([]);
    expect(result.failed).toEqual([]);
    expect(result.imported).toHaveLength(2);
    expect(result.imported.every((game) => game.isPrimaryTeamGame)).toBe(true);
    expect(imports.records).toHaveLength(2);
    expect(result.imported[0]?.fixtureId).toBe(result.imported[1]?.fixtureId);
  });

  it('reports and merges conflicting parameters with field choices', async () => {
    const games = new InMemoryGameRepository();
    const imports = new InMemoryGameImportRepository();
    const record = {
      arrivalBufferMinutes: 45,
      awayTeamName: 'U18-1',
      date: '2026-08-15',
      homeTeamName: 'U16-1',
      locationName: 'New court',
      sourceRow: 2,
      startTime: '14:30',
      travelMinutes: 35,
    };
    const original = await importGames(games, imports, {
      importedAt: new Date('2026-08-01T10:00:00.000Z'),
      primaryTeamName: 'U16-1',
      records: [
        { ...record, arrivalBufferMinutes: 30, locationName: 'Old court', travelMinutes: 20 },
      ],
      sourceName: 'schedule.csv',
    });
    const conflicts = await findImportConflicts(games, [record]);

    expect(conflicts.conflicts[0]?.fields).toHaveLength(3);
    const result = await importGames(games, imports, {
      importedAt: new Date('2026-08-02T10:00:00.000Z'),
      primaryTeamName: 'U16-1',
      records: [record],
      resolutions: [
        {
          fields: {
            arrivalBufferMinutes: 'imported',
            locationName: 'existing',
            travelMinutes: 'imported',
          },
          sourceRow: 2,
        },
      ],
      sourceName: 'schedule.csv',
    });

    expect(original.imported).toHaveLength(1);
    expect(result.imported[0]).toMatchObject({
      arrivalBufferMinutes: 45,
      locationName: 'Old court',
      merged: true,
      travelMinutes: 35,
    });
  });

  it('skips an exact duplicate on a repeated import', async () => {
    const games = new InMemoryGameRepository();
    const imports = new InMemoryGameImportRepository();
    const input = {
      arrivalBufferMinutes: 30,
      awayTeamName: 'U18-1',
      date: '2026-08-15',
      homeTeamName: 'U16-1',
      locationName: 'Away court',
      sourceRow: 2,
      startTime: '14:30',
      travelMinutes: 20,
    };

    await importGames(games, imports, {
      importedAt: new Date('2026-08-01T10:00:00.000Z'),
      primaryTeamName: 'U16-1',
      records: [input],
      sourceName: 'schedule.csv',
    });
    const result = await importGames(games, imports, {
      importedAt: new Date('2026-08-02T10:00:00.000Z'),
      primaryTeamName: 'U16-1',
      records: [input],
      sourceName: 'schedule.csv',
    });

    expect(result.imported).toEqual([]);
    expect(result.duplicates).toHaveLength(1);
    expect(imports.records).toHaveLength(1);
  });
});
