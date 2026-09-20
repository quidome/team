import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  duties: {
    assign: vi.fn(),
    configure: vi.fn(),
    findAllSlots: vi.fn(),
    findByOccurrence: vi.fn(),
    findFairness: vi.fn(),
    recordSignup: vi.fn(),
    updateSlotStatus: vi.fn(),
  },
  games: {
    findAllOccurrences: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentDutyRepository: () => mocks.duties,
  currentGameRepository: () => mocks.games,
}));

import { GET, POST } from '../../../routes/api/duties/+server';

describe('/api/duties', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.games.findAllOccurrences.mockResolvedValue([]);
    mocks.duties.findAllSlots.mockResolvedValue([]);
    mocks.duties.findByOccurrence.mockResolvedValue({
      assignmentHistory: [],
      fairness: [],
      occurrenceId: 'game-occurrence-1',
      requirements: { drivingSlots: 0, jurySlots: 1, refereeSlots: 2 },
      signups: [],
      slots: [],
    });
    mocks.duties.configure.mockResolvedValue({
      occurrenceId: 'game-occurrence-1',
      requirements: { drivingSlots: 1, jurySlots: 0, refereeSlots: 2 },
      assignmentHistory: [],
      fairness: [],
      signups: [],
      slots: [],
    });
  });

  it('stores per-game duty requirements', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/duties', {
        body: JSON.stringify({
          drivingSlots: 1,
          jurySlots: 0,
          occurrenceId: 'game-occurrence-1',
          refereeSlots: 2,
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.duties.configure).toHaveBeenCalledWith('game-occurrence-1', {
      drivingSlots: 1,
      jurySlots: 0,
      refereeSlots: 2,
    });
  });

  it('rejects more than two slots of one duty type', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/duties', {
        body: JSON.stringify({
          drivingSlots: 3,
          jurySlots: 0,
          occurrenceId: 'game-occurrence-1',
          refereeSlots: 0,
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_duty_requirements' });
    expect(mocks.duties.configure).not.toHaveBeenCalled();
  });

  it('loads duties for one game and completes due assignments first', async () => {
    const response = await GET({
      url: new URL('http://localhost/api/duties?occurrenceId=game-occurrence-1'),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.duties.findAllSlots).toHaveBeenCalledOnce();
    expect(mocks.duties.findByOccurrence).toHaveBeenCalledWith('game-occurrence-1');
  });
});
