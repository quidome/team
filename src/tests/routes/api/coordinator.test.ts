import { beforeEach, describe, expect, it, vi } from 'vitest';

import { InMemoryCoordinatorRepository } from '$lib/adapters/in-memory-coordinator-repository';
import { InMemoryTeamRepository } from '$lib/adapters/in-memory-team-repository';

const mocks = vi.hoisted(() => ({
  audit: { record: vi.fn() },
  repositories: {} as {
    coordinators: InMemoryCoordinatorRepository;
    teams: InMemoryTeamRepository;
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentCoordinatorRepository: () => mocks.repositories.coordinators,
  currentTeamRepository: () => mocks.repositories.teams,
}));

import { DELETE, GET, PATCH, POST, PUT } from '../../../routes/api/coordinator/+server';

const subject = 'pocket-id-subject-sam';
const locals = { coordinatorSession: { subject } };

const jsonRequest = (method: string, body: unknown) =>
  new Request('http://localhost/api/coordinator', {
    body: JSON.stringify(body),
    headers: { 'content-type': 'application/json' },
    method,
  });

describe('/api/coordinator', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.repositories.coordinators = new InMemoryCoordinatorRepository();
    mocks.repositories.teams = new InMemoryTeamRepository([
      { isOwnTeam: true, name: 'U16-1' },
      { isOwnTeam: true, name: 'U16-2' },
    ]);
  });

  it('returns the current coordinator, storing it when it is new', async () => {
    const response = await GET({ locals } as never);

    await expect(response.json()).resolves.toEqual({ subject, teamNames: [] });
  });

  it('takes responsibility for a team and sets it as the default team', async () => {
    await POST({ locals, request: jsonRequest('POST', { teamName: 'U16-1' }) } as never);
    const response = await PUT({
      locals,
      request: jsonRequest('PUT', { teamName: 'U16-1' }),
    } as never);

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual({
      defaultTeamName: 'U16-1',
      subject,
      teamNames: ['U16-1'],
    });
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'coordinator_default_team_set', entityId: subject }),
    );
  });

  it('rejects a default team the coordinator is not responsible for', async () => {
    const response = await PUT({
      locals,
      request: jsonRequest('PUT', { teamName: 'U16-2' }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({
      error: 'Coordinator is not responsible for U16-2',
    });
  });

  it('rejects responsibility for an unknown team', async () => {
    const response = await POST({
      locals,
      request: jsonRequest('POST', { teamName: 'U20-1' }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'Team U20-1 does not exist' });
  });

  it('gives up responsibility for a team', async () => {
    await POST({ locals, request: jsonRequest('POST', { teamName: 'U16-1' }) } as never);

    const response = await DELETE({
      locals,
      url: new URL('http://localhost/api/coordinator?teamName=U16-1'),
    } as never);

    await expect(response.json()).resolves.toEqual({ subject, teamNames: [] });
  });

  it('updates the coordinator profile', async () => {
    const response = await PATCH({
      locals,
      request: jsonRequest('PATCH', { displayName: 'Sam Jansen', email: 'sam@example.test' }),
    } as never);

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual({
      profile: { displayName: 'Sam Jansen', email: 'sam@example.test' },
      subject,
      teamNames: [],
    });
  });

  it('rejects requests without a coordinator session', async () => {
    const response = await GET({ locals: {} } as never);

    expect(response.status).toBe(401);
  });
});
