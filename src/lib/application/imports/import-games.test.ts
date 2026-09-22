import { describe, expect, it, vi } from 'vitest';

import { InMemoryDutyRepository } from '../../adapters/in-memory-duty-repository';
import { InMemoryGameImportRepository } from '../../adapters/in-memory-game-import-repository';
import { InMemoryGameRepository } from '../../adapters/in-memory-game-repository';
import { findImportConflicts, importGames } from './import-games';

const knownTeamsContext = { knownTeamNames: ['U16-1', 'U18-1'] };

describe('importGames', () => {
  it('imports all valid games, reuses fixtures, classifies primary-team games, and stores provenance', async () => {
    const games = new InMemoryGameRepository();
    const imports = new InMemoryGameImportRepository();
    const duties = new InMemoryDutyRepository();
    const result = await importGames(games, imports, duties, {
      context: knownTeamsContext,
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
    const duties = new InMemoryDutyRepository();
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
    const original = await importGames(games, imports, duties, {
      context: knownTeamsContext,
      importedAt: new Date('2026-08-01T10:00:00.000Z'),
      primaryTeamName: 'U16-1',
      records: [
        { ...record, arrivalBufferMinutes: 30, locationName: 'Old court', travelMinutes: 20 },
      ],
      sourceName: 'schedule.csv',
    });
    const conflicts = await findImportConflicts(games, [record]);

    expect(conflicts.conflicts[0]?.fields).toHaveLength(3);
    const result = await importGames(games, imports, duties, {
      context: knownTeamsContext,
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

  it('fails fast in atomic mode when provenance cannot be stored', async () => {
    const games = new InMemoryGameRepository();
    const imports = {
      save: async () => {
        throw new Error('provenance storage failed');
      },
    };
    const duties = new InMemoryDutyRepository();

    await expect(
      importGames(games, imports, duties, {
        atomic: true,
        context: knownTeamsContext,
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
        ],
        sourceName: 'schedule.csv',
      }),
    ).rejects.toThrow('provenance storage failed');
  });

  it('skips an exact duplicate on a repeated import', async () => {
    const games = new InMemoryGameRepository();
    const imports = new InMemoryGameImportRepository();
    const duties = new InMemoryDutyRepository();
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

    await importGames(games, imports, duties, {
      context: knownTeamsContext,
      importedAt: new Date('2026-08-01T10:00:00.000Z'),
      primaryTeamName: 'U16-1',
      records: [input],
      sourceName: 'schedule.csv',
    });
    const result = await importGames(games, imports, duties, {
      context: knownTeamsContext,
      importedAt: new Date('2026-08-02T10:00:00.000Z'),
      primaryTeamName: 'U16-1',
      records: [input],
      sourceName: 'schedule.csv',
    });

    expect(result.imported).toEqual([]);
    expect(result.duplicates).toHaveLength(1);
    expect(imports.records).toHaveLength(1);
  });

  it('resolves a row with one recognized team as an opponent fixture', async () => {
    const games = new InMemoryGameRepository();
    const imports = new InMemoryGameImportRepository();
    const duties = new InMemoryDutyRepository();
    const result = await importGames(games, imports, duties, {
      context: { knownTeamNames: ['U16-1'], season: { startingYear: 2026 } },
      importedAt: new Date('2026-08-01T10:00:00.000Z'),
      primaryTeamName: 'U16-1',
      records: [
        {
          arrivalBufferMinutes: 30,
          awayTeamName: 'U16-1',
          date: '2026-09-26',
          homeTeamName: 'Woodpeckers M16-2',
          locationName: 'Away court',
          sourceRow: 2,
          startTime: '16:45',
          travelMinutes: 35,
        },
      ],
      sourceName: 'schedule.csv',
    });

    expect(result.failed).toEqual([]);
    expect(result.imported).toHaveLength(1);
    const [fixtureId] = result.imported.map((game) => game.fixtureId);
    const stored = await games.findFixtureById(fixtureId as string);
    expect(stored?.fixture).toMatchObject({
      isHome: false,
      opponentName: 'Woodpeckers M16-2',
      ourTeamName: 'U16-1',
      seasonHalf: 'H1',
    });
  });

  it('fails a row where neither team is recognized', async () => {
    const games = new InMemoryGameRepository();
    const imports = new InMemoryGameImportRepository();
    const duties = new InMemoryDutyRepository();
    const result = await importGames(games, imports, duties, {
      context: knownTeamsContext,
      importedAt: new Date('2026-08-01T10:00:00.000Z'),
      primaryTeamName: 'U16-1',
      records: [
        {
          arrivalBufferMinutes: 30,
          awayTeamName: 'Archipel M16-1',
          date: '2026-09-26',
          homeTeamName: 'Woodpeckers M16-2',
          locationName: 'Away court',
          sourceRow: 2,
          startTime: '16:45',
          travelMinutes: 35,
        },
      ],
      sourceName: 'schedule.csv',
    });

    expect(result.imported).toEqual([]);
    expect(result.failed).toEqual([
      {
        message: 'Neither team is recognized — add one as a team in Admin first.',
        sourceRow: 2,
      },
    ]);
  });

  it('fails an opponent row whose date falls outside the configured season', async () => {
    const games = new InMemoryGameRepository();
    const imports = new InMemoryGameImportRepository();
    const duties = new InMemoryDutyRepository();
    const result = await importGames(games, imports, duties, {
      context: { knownTeamNames: ['U16-1'], season: { startingYear: 2026 } },
      importedAt: new Date('2026-08-01T10:00:00.000Z'),
      primaryTeamName: 'U16-1',
      records: [
        {
          arrivalBufferMinutes: 30,
          awayTeamName: 'U16-1',
          date: '2028-01-09',
          homeTeamName: 'Woodpeckers M16-2',
          locationName: 'Away court',
          sourceRow: 2,
          startTime: '16:45',
          travelMinutes: 35,
        },
      ],
      sourceName: 'schedule.csv',
    });

    expect(result.imported).toEqual([]);
    expect(result.failed).toEqual([
      { message: 'Game date falls outside the configured season.', sourceRow: 2 },
    ]);
  });

  it('separates opponent games in different calendar years of the season into distinct fixtures', async () => {
    const games = new InMemoryGameRepository();
    const imports = new InMemoryGameImportRepository();
    const duties = new InMemoryDutyRepository();
    const result = await importGames(games, imports, duties, {
      context: { knownTeamNames: ['U16-1'], season: { startingYear: 2026 } },
      importedAt: new Date('2026-08-01T10:00:00.000Z'),
      primaryTeamName: 'U16-1',
      records: [
        {
          arrivalBufferMinutes: 30,
          awayTeamName: 'U16-1',
          date: '2026-09-26',
          homeTeamName: 'Woodpeckers M16-2',
          locationName: 'Away court',
          sourceRow: 2,
          startTime: '16:45',
          travelMinutes: 35,
        },
        {
          arrivalBufferMinutes: 30,
          awayTeamName: 'U16-1',
          date: '2027-01-09',
          homeTeamName: 'Woodpeckers M16-2',
          locationName: 'Away court',
          sourceRow: 3,
          startTime: '14:00',
          travelMinutes: 35,
        },
      ],
      sourceName: 'schedule.csv',
    });

    expect(result.failed).toEqual([]);
    expect(result.imported).toHaveLength(2);
    expect(result.imported[0]?.fixtureId).not.toBe(result.imported[1]?.fixtureId);
  });

  it('configures duty requirements when jury or referee slots are present, for new and merged occurrences', async () => {
    const games = new InMemoryGameRepository();
    const imports = new InMemoryGameImportRepository();
    const duties = new InMemoryDutyRepository();
    const configureSpy = vi.spyOn(duties, 'configure');
    const record = {
      arrivalBufferMinutes: 30,
      awayTeamName: 'U18-1',
      date: '2026-08-15',
      homeTeamName: 'U16-1',
      jurySlots: 2,
      locationName: 'Away court',
      refereeSlots: 1,
      sourceRow: 2,
      startTime: '14:30',
      travelMinutes: 20,
    };
    const created = await importGames(games, imports, duties, {
      context: knownTeamsContext,
      importedAt: new Date('2026-08-01T10:00:00.000Z'),
      primaryTeamName: 'U16-1',
      records: [record],
      sourceName: 'schedule.csv',
    });
    const occurrenceId = created.imported[0]?.occurrenceId as string;

    expect(configureSpy).toHaveBeenCalledWith(occurrenceId, {
      drivingSlots: 0,
      jurySlots: 2,
      refereeSlots: 1,
    });

    configureSpy.mockClear();
    const conflictingRecord = { ...record, jurySlots: 1, refereeSlots: 2, travelMinutes: 25 };
    const conflicts = await findImportConflicts(games, [conflictingRecord]);
    const merged = await importGames(games, imports, duties, {
      context: knownTeamsContext,
      importedAt: new Date('2026-08-02T10:00:00.000Z'),
      primaryTeamName: 'U16-1',
      records: [conflictingRecord],
      resolutions: conflicts.conflicts.map((conflict) => ({
        fields: { travelMinutes: 'imported' },
        sourceRow: conflict.sourceRow,
      })),
      sourceName: 'schedule.csv',
    });

    expect(merged.imported[0]?.merged).toBe(true);
    expect(configureSpy).toHaveBeenCalledWith(occurrenceId, {
      drivingSlots: 0,
      jurySlots: 1,
      refereeSlots: 2,
    });
  });

  it('does not configure duty requirements when jury and referee slots are blank', async () => {
    const games = new InMemoryGameRepository();
    const imports = new InMemoryGameImportRepository();
    const duties = new InMemoryDutyRepository();
    const configureSpy = vi.spyOn(duties, 'configure');

    await importGames(games, imports, duties, {
      context: knownTeamsContext,
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
      ],
      sourceName: 'schedule.csv',
    });

    expect(configureSpy).not.toHaveBeenCalled();
  });
});
