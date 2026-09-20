import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  audit: {
    record: vi.fn(),
  },
  participation: {
    findByOccurrence: vi.fn(),
    saveMany: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentParticipationRepository: () => mocks.participation,
}));

import { GET, POST } from '../../../routes/api/participation/+server';

describe('POST /api/participation', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.participation.saveMany.mockImplementation(async (records) => records);
  });

  it('loads attendance for an occurrence', async () => {
    mocks.participation.findByOccurrence.mockResolvedValue([
      {
        occurrenceId: 'training-occurrence-1',
        occurrenceType: 'training',
        playerAssociationId: 'avery',
        status: 'present',
      },
    ]);

    const response = await GET({
      url: new URL(
        'http://localhost/api/participation?occurrenceType=training&occurrenceId=training-occurrence-1',
      ),
    } as never);

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual([
      expect.objectContaining({ playerAssociationId: 'avery', status: 'present' }),
    ]);
  });

  it('records a bulk training attendance selection', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/participation', {
        body: JSON.stringify({
          absences: [{ playerAssociationId: 'blake', reason: 'illness' }],
          eligiblePlayerAssociationIds: ['avery', 'blake', 'casey'],
          occurrenceId: 'training-occurrence-1',
          occurrenceType: 'training',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'attendance_recorded', entityType: 'participation' }),
    );
    await expect(response.json()).resolves.toEqual([
      expect.objectContaining({ playerAssociationId: 'avery', status: 'present' }),
      expect.objectContaining({
        absenceReason: 'illness',
        playerAssociationId: 'blake',
        status: 'absent',
      }),
      expect.objectContaining({ playerAssociationId: 'casey', status: 'present' }),
    ]);
  });

  it('rejects an invalid occurrence type', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/participation', {
        body: JSON.stringify({
          absences: [],
          eligiblePlayerAssociationIds: ['avery'],
          occurrenceId: 'training-occurrence-1',
          occurrenceType: 'unknown',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_participation' });
    expect(mocks.participation.saveMany).not.toHaveBeenCalled();
  });
});
