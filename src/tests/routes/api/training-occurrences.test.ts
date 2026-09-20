import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  audit: {
    record: vi.fn(),
  },
  trainingSeries: {
    findOccurrenceById: vi.fn(),
    saveOccurrence: vi.fn(),
    updateOccurrenceStatus: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentTrainingSeriesRepository: () => mocks.trainingSeries,
}));

import { POST } from '../../../routes/api/training-occurrences/+server';
import { POST as POST_CANCEL } from '../../../routes/api/training-occurrences/[occurrenceId]/cancel/+server';
import { POST as POST_RESCHEDULE } from '../../../routes/api/training-occurrences/[occurrenceId]/reschedule/+server';

describe('POST /api/training-occurrences', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.trainingSeries.saveOccurrence.mockImplementation(async (occurrence) => ({
      ...occurrence,
      id: 'training-occurrence-1',
      status: 'scheduled',
    }));
    mocks.trainingSeries.findOccurrenceById.mockResolvedValue({
      date: '2026-08-20',
      durationMinutes: 60,
      id: 'training-occurrence-1',
      locationName: 'Away court',
      startTime: '17:00',
      status: 'scheduled',
    });
    mocks.trainingSeries.updateOccurrenceStatus.mockImplementation(async (id, status) => ({
      date: '2026-08-20',
      durationMinutes: 60,
      id,
      locationName: 'Away court',
      startTime: '17:00',
      status,
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

  it('cancels an occurrence', async () => {
    const response = await POST_CANCEL({
      params: { occurrenceId: 'training-occurrence-1' },
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.trainingSeries.updateOccurrenceStatus).toHaveBeenCalledWith(
      'training-occurrence-1',
      'cancelled',
    );
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({
        action: 'training_occurrence_cancelled',
        entityId: 'training-occurrence-1',
      }),
    );
    await expect(response.json()).resolves.toMatchObject({ status: 'cancelled' });
  });

  it('reschedules an occurrence and cancels its original', async () => {
    const response = await POST_RESCHEDULE({
      params: { occurrenceId: 'training-occurrence-1' },
      request: new Request(
        'http://localhost/api/training-occurrences/training-occurrence-1/reschedule',
        {
          body: JSON.stringify({
            date: '2026-08-27',
            durationMinutes: 75,
            locationName: 'Home court',
            startTime: '19:00',
          }),
          headers: { 'content-type': 'application/json' },
          method: 'POST',
        },
      ),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.trainingSeries.saveOccurrence).toHaveBeenCalledWith({
      date: '2026-08-27',
      durationMinutes: 75,
      locationName: 'Home court',
      startTime: '19:00',
      status: 'scheduled',
    });
    expect(mocks.trainingSeries.updateOccurrenceStatus).toHaveBeenCalledWith(
      'training-occurrence-1',
      'cancelled',
    );
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'training_occurrence_rescheduled' }),
    );
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
