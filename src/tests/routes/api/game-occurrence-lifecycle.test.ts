import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  games: {
    findFixtureById: vi.fn(),
    findOccurrenceById: vi.fn(),
    findOccurrences: vi.fn(),
    saveFixture: vi.fn(),
    saveOccurrence: vi.fn(),
    updateOccurrenceStatus: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentGameRepository: () => mocks.games,
}));

import { POST as cancel } from '../../../routes/api/game-occurrences/[occurrenceId]/cancel/+server';
import { POST as reschedule } from '../../../routes/api/game-occurrences/[occurrenceId]/reschedule/+server';

describe('game occurrence lifecycle', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.games.findOccurrenceById.mockResolvedValue({
      arrivalBufferMinutes: 30,
      date: '2026-09-05',
      fixtureId: 'game-fixture-1',
      id: 'game-occurrence-1',
      locationName: 'Home court',
      startTime: '14:30',
      status: 'scheduled',
      suggestedDepartureTime: '14:00',
      travelMinutes: 0,
    });
    mocks.games.updateOccurrenceStatus.mockResolvedValue({
      id: 'game-occurrence-1',
      status: 'cancelled',
    });
    mocks.games.saveOccurrence.mockResolvedValue({
      date: '2026-09-12',
      fixtureId: 'game-fixture-1',
      id: 'game-occurrence-2',
      status: 'scheduled',
    });
  });

  it('cancels an occurrence without deleting it', async () => {
    await expect(
      cancel({ params: { occurrenceId: 'game-occurrence-1' } } as never),
    ).resolves.toMatchObject({
      status: 200,
    });
    expect(mocks.games.updateOccurrenceStatus).toHaveBeenCalledWith(
      'game-occurrence-1',
      'cancelled',
    );
  });

  it('creates a replacement and cancels the original when rescheduling', async () => {
    const response = await reschedule({
      params: { occurrenceId: 'game-occurrence-1' },
      request: new Request('http://localhost/api/game-occurrences/game-occurrence-1/reschedule', {
        body: JSON.stringify({
          arrivalBufferMinutes: 30,
          date: '2026-09-12',
          locationName: 'Home court',
          startTime: '15:00',
          travelMinutes: 0,
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.games.saveOccurrence).toHaveBeenCalledWith(
      'game-fixture-1',
      expect.objectContaining({ date: '2026-09-12', startTime: '15:00' }),
    );
    expect(mocks.games.updateOccurrenceStatus).toHaveBeenCalledWith(
      'game-occurrence-1',
      'cancelled',
    );
  });
});
