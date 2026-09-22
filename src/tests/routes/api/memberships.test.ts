import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  audit: {
    record: vi.fn(),
  },
  memberships: {
    find: vi.fn(),
    save: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentMembershipRepository: () => mocks.memberships,
}));

import { POST } from '../../../routes/api/memberships/+server';

describe('POST /api/memberships', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.memberships.find.mockResolvedValue(undefined);
    mocks.memberships.save.mockImplementation(async (membership) => membership);
  });

  it('records an active primary membership', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/memberships', {
        body: JSON.stringify({
          jerseyNumber: 7,
          participationType: 'trains_and_plays',
          playerId: '12345',
          relationship: 'primary',
          seasonStartingYear: 2026,
          status: 'active',
          teamName: 'U16-1',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'membership_configured', entityType: 'membership' }),
    );
    await expect(response.json()).resolves.toEqual({
      jerseyNumber: 7,
      participationType: 'trains_and_plays',
      playerId: '12345',
      relationship: 'primary',
      seasonStartingYear: 2026,
      status: 'active',
      teamName: 'U16-1',
    });
  });

  it('rejects an unsupported participation type', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/memberships', {
        body: JSON.stringify({
          participationType: 'plays_only',
          playerId: '12345',
          relationship: 'primary',
          seasonStartingYear: 2026,
          status: 'active',
          teamName: 'U16-1',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_membership' });
    expect(mocks.memberships.save).not.toHaveBeenCalled();
  });
});
