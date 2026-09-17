import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  games: {
    findFixtureById: vi.fn(),
    findOccurrences: vi.fn(),
    saveFixture: vi.fn(),
    saveOccurrence: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentGameRepository: () => mocks.games,
}));

import { POST } from '../../../routes/api/game-occurrences/+server';

describe('POST /api/game-occurrences', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.games.saveOccurrence.mockImplementation(async (fixtureId, occurrence) => ({
      ...occurrence,
      fixtureId,
      id: 'game-occurrence-1',
      status: 'scheduled',
    }));
  });

  it('schedules an occurrence with its arrival buffer', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/game-occurrences', {
        body: JSON.stringify({
          arrivalBufferMinutes: 30,
          date: '2026-09-05',
          fixtureId: 'game-fixture-1',
          locationName: 'Home court',
          startTime: '14:30',
          travelMinutes: 0,
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toMatchObject({
      date: '2026-09-05',
      fixtureId: 'game-fixture-1',
      status: 'scheduled',
    });
  });

  it('rejects an invalid game time', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/game-occurrences', {
        body: JSON.stringify({
          arrivalBufferMinutes: 30,
          date: '2026-09-05',
          fixtureId: 'game-fixture-1',
          locationName: 'Home court',
          startTime: '14:70',
          travelMinutes: 0,
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_game_occurrence' });
    expect(mocks.games.saveOccurrence).not.toHaveBeenCalled();
  });
});
