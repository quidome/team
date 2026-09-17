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

import { POST } from '../../../routes/api/games/fixtures/+server';

describe('POST /api/games/fixtures', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.games.saveFixture.mockImplementation(async (fixture) => ({
      fixture,
      id: 'game-fixture-1',
    }));
  });

  it('configures a stable home/away fixture', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/games/fixtures', {
        body: JSON.stringify({ awayTeamName: 'U18-1', homeTeamName: 'U16-1' }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual({
      fixture: { awayTeamName: 'U18-1', homeTeamName: 'U16-1' },
      id: 'game-fixture-1',
    });
  });
});
