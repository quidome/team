import { describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  settings: {
    get: vi.fn(),
  },
  teams: {
    findAll: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentCoordinatorSettingsRepository: () => mocks.settings,
  currentTeamRepository: () => mocks.teams,
}));

mocks.settings.get.mockResolvedValue({ primaryTeamName: 'U16-1', seasonStartingYear: 2026 });
mocks.teams.findAll.mockResolvedValue([{ name: 'U16-1' }, { name: 'U18-1' }]);

import { POST } from '../../../routes/api/imports/games/preview/+server';

describe('POST /api/imports/games/preview', () => {
  it('returns a validated CSV preview', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/imports/games/preview', {
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
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual(
      expect.objectContaining({ issues: [], validRowCount: 1 }),
    );
  });

  it('rejects malformed preview requests', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/imports/games/preview', {
        body: JSON.stringify({
          content: 'home,away',
          encoding: 'text',
          fileName: 'schedule.csv',
          mapping: 'invalid',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_import_preview' });
  });
});
