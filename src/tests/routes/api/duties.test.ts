import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  audit: {
    record: vi.fn(),
  },
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
  currentAuditRepository: () => mocks.audit,
  currentDutyRepository: () => mocks.duties,
  currentGameRepository: () => mocks.games,
}));

import { GET, POST } from '../../../routes/api/duties/+server';
import { POST as assign } from '../../../routes/api/duties/assignments/+server';
import { POST as signup } from '../../../routes/api/duties/signups/+server';
import { POST as updateStatus } from '../../../routes/api/duties/slots/[slotId]/status/+server';

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
    mocks.duties.assign.mockResolvedValue({
      assignmentHistory: [],
      fairness: [],
      occurrenceId: 'game-occurrence-1',
      requirements: { drivingSlots: 1, jurySlots: 0, refereeSlots: 2 },
      signups: [],
      slots: [],
    });
    mocks.duties.recordSignup.mockResolvedValue({
      assignmentHistory: [],
      fairness: [],
      occurrenceId: 'game-occurrence-1',
      requirements: { drivingSlots: 1, jurySlots: 0, refereeSlots: 2 },
      signups: [],
      slots: [],
    });
    mocks.duties.updateSlotStatus.mockResolvedValue({
      assignmentHistory: [],
      fairness: [],
      occurrenceId: 'game-occurrence-1',
      requirements: { drivingSlots: 1, jurySlots: 0, refereeSlots: 2 },
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
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({
        action: 'duty_requirements_configured',
        entityId: 'game-occurrence-1',
      }),
    );
  });

  it('records a duty assignment', async () => {
    const response = await assign({
      request: new Request('http://localhost/api/duties/assignments', {
        body: JSON.stringify({ playerAssociationId: 'player-1', slotId: 'slot-1' }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.duties.assign).toHaveBeenCalledWith('slot-1', 'player-1');
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'duty_assigned', entityId: 'slot-1' }),
    );
  });

  it('records a duty signup', async () => {
    const response = await signup({
      request: new Request('http://localhost/api/duties/signups', {
        body: JSON.stringify({
          dutyType: 'driving',
          occurrenceId: 'game-occurrence-1',
          playerAssociationId: 'player-1',
          status: 'volunteer',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'duty_signup_recorded' }),
    );
  });

  it('records a duty status correction', async () => {
    const response = await updateStatus({
      params: { slotId: 'slot-1' },
      request: new Request('http://localhost/api/duties/slots/slot-1/status', {
        body: JSON.stringify({ status: 'completed' }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'duty_status_changed', entityId: 'slot-1' }),
    );
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
