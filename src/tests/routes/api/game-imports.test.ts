import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  audit: {
    record: vi.fn(),
  },
  games: {
    findAllOccurrences: vi.fn(),
    saveFixture: vi.fn(),
    saveOccurrence: vi.fn(),
    updateOccurrence: vi.fn(),
  },
  imports: {
    save: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentGameImportRepository: () => mocks.imports,
  currentGameRepository: () => mocks.games,
}));

import { POST } from '../../../routes/api/imports/games/+server';

describe('POST /api/imports/games', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.games.findAllOccurrences.mockResolvedValue([]);
    mocks.games.saveFixture.mockResolvedValue({
      fixture: { awayTeamName: 'U18-1', homeTeamName: 'U16-1' },
      id: 'fixture-1',
    });
    mocks.games.saveOccurrence.mockResolvedValue({
      arrivalBufferMinutes: 30,
      date: '2026-08-15',
      fixtureId: 'fixture-1',
      id: 'occurrence-1',
      locationName: 'Away court',
      startTime: '14:30',
      status: 'scheduled',
      suggestedDepartureTime: '13:40',
      travelMinutes: 20,
    });
    mocks.imports.save.mockImplementation(async (provenance) => ({
      ...provenance,
      id: 'provenance-1',
    }));
    mocks.games.updateOccurrence.mockResolvedValue({
      arrivalBufferMinutes: 45,
      date: '2026-08-15',
      fixtureId: 'fixture-existing',
      id: 'occurrence-existing',
      locationName: 'New court',
      startTime: '14:30',
      status: 'scheduled',
      suggestedDepartureTime: '13:40',
      travelMinutes: 35,
    });
  });

  it('imports a valid mapped CSV and stores provenance', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/imports/games', {
        body: JSON.stringify({
          content: 'home,away,date,time,location\nU16-1,U18-1,2026-08-15,14:30,Away court',
          encoding: 'text',
          fileName: 'schedule.csv',
          mapping: {
            awayTeamName: 'away',
            date: 'date',
            homeTeamName: 'home',
            locationName: 'location',
            startTime: 'time',
          },
          primaryTeamName: 'U16-1',
          sourceName: 'schedule.csv',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual({
      conflicts: [],
      duplicates: [],
      failed: [],
      imported: [
        expect.objectContaining({
          isPrimaryTeamGame: true,
          occurrenceId: 'occurrence-1',
          sourceRow: 2,
        }),
      ],
    });
    expect(mocks.imports.save).toHaveBeenCalledWith(
      expect.objectContaining({ occurrenceId: 'occurrence-1', sourceName: 'schedule.csv' }),
    );
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'games_imported', entityId: 'schedule.csv' }),
    );
  });

  it('returns field-level conflicts before changing an existing game', async () => {
    mocks.games.findAllOccurrences.mockResolvedValue([
      {
        arrivalBufferMinutes: 30,
        awayTeamName: 'U18-1',
        date: '2026-08-15',
        fixtureId: 'fixture-existing',
        homeTeamName: 'U16-1',
        id: 'occurrence-existing',
        locationName: 'Old court',
        startTime: '14:30',
        status: 'scheduled',
        suggestedDepartureTime: '13:50',
        travelMinutes: 20,
      },
    ]);

    const response = await POST({
      request: new Request('http://localhost/api/imports/games', {
        body: JSON.stringify({
          content: 'home,away,date,time,location,travel\nU16-1,U18-1,2026-08-15,14:30,New court,35',
          encoding: 'text',
          fileName: 'schedule.csv',
          mapping: {
            awayTeamName: 'away',
            date: 'date',
            homeTeamName: 'home',
            locationName: 'location',
            startTime: 'time',
            travelMinutes: 'travel',
          },
          primaryTeamName: 'U16-1',
          sourceName: 'schedule.csv',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(409);
    await expect(response.json()).resolves.toMatchObject({
      error: 'import_conflicts',
      conflicts: [
        {
          existingOccurrenceId: 'occurrence-existing',
          sourceRow: 2,
        },
      ],
    });
    expect(mocks.games.updateOccurrence).not.toHaveBeenCalled();
  });

  it('does not import rows that still have validation issues', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/imports/games', {
        body: JSON.stringify({
          content: 'home,away,date,time,location\nU16-1,U18-1,not-a-date,14:30,Away court',
          encoding: 'text',
          fileName: 'schedule.csv',
          mapping: {
            awayTeamName: 'away',
            date: 'date',
            homeTeamName: 'home',
            locationName: 'location',
            startTime: 'time',
          },
          primaryTeamName: 'U16-1',
          sourceName: 'schedule.csv',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(400);
    expect(mocks.games.saveFixture).not.toHaveBeenCalled();
  });
});
