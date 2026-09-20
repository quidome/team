import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  audit: {
    record: vi.fn(),
  },
  trainingSeries: {
    saveOccurrence: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentTrainingSeriesRepository: () => mocks.trainingSeries,
}));

import { POST } from '../../../routes/api/training-occurrences/+server';

describe('POST /api/training-occurrences', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.trainingSeries.saveOccurrence.mockImplementation(async (occurrence) => ({
      ...occurrence,
      id: 'training-occurrence-1',
    }));
  });

  it('stores a one-off training occurrence', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/training-occurrences', {
        body: JSON.stringify({
          date: '2026-08-20',
          durationMinutes: 60,
          locationName: 'Away court',
          startTime: '17:00',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.trainingSeries.saveOccurrence).toHaveBeenCalledWith({
      date: '2026-08-20',
      durationMinutes: 60,
      locationName: 'Away court',
      startTime: '17:00',
    });
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({
        action: 'training_occurrence_created',
        entityId: 'training-occurrence-1',
      }),
    );
    await expect(response.json()).resolves.toMatchObject({ id: 'training-occurrence-1' });
  });

  it('rejects an invalid date', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/training-occurrences', {
        body: JSON.stringify({
          date: 'not-a-date',
          durationMinutes: 60,
          locationName: 'Away court',
          startTime: '17:00',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_training_occurrence' });
    expect(mocks.trainingSeries.saveOccurrence).not.toHaveBeenCalled();
  });
});
