import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  audit: {
    record: vi.fn(),
  },
  teams: {
    findByName: vi.fn(),
    save: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentTeamRepository: () => mocks.teams,
}));

import { POST } from '../../../routes/api/teams/+server';

describe('POST /api/teams', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.teams.findByName.mockResolvedValue(undefined);
    mocks.teams.save.mockImplementation(async (team) => team);
  });

  it('configures a team by name', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/teams', {
        body: JSON.stringify({ name: 'U16-1' }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'team_configured', entityId: 'U16-1' }),
    );
    await expect(response.json()).resolves.toEqual({ isOwnTeam: true, name: 'U16-1' });
    expect(mocks.teams.save).toHaveBeenCalledWith({ isOwnTeam: true, name: 'U16-1' });
  });

  it('rejects a blank team name', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/teams', {
        body: JSON.stringify({ name: '   ' }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_team_name' });
    expect(mocks.teams.save).not.toHaveBeenCalled();
  });
});
