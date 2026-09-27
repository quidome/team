import { beforeEach, describe, expect, it, vi } from 'vitest';

import { InMemoryCoordinatorRepository } from '$lib/adapters/in-memory-coordinator-repository';
import { InMemoryCoordinatorSettingsRepository } from '$lib/adapters/in-memory-coordinator-settings-repository';
import { InMemorySeasonRepository } from '$lib/adapters/in-memory-season-repository';
import { InMemoryTeamRepository } from '$lib/adapters/in-memory-team-repository';

const mocks = vi.hoisted(() => ({
  audit: { record: vi.fn() },
  repositories: {} as {
    coordinators: InMemoryCoordinatorRepository;
    seasons: InMemorySeasonRepository;
    settings: InMemoryCoordinatorSettingsRepository;
    teams: InMemoryTeamRepository;
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentCoordinatorRepository: () => mocks.repositories.coordinators,
  currentCoordinatorSettingsRepository: () => mocks.repositories.settings,
  currentSeasonRepository: () => mocks.repositories.seasons,
  currentTeamRepository: () => mocks.repositories.teams,
}));

import { PUT } from '../../../routes/api/settings/+server';

const subject = 'pocket-id-subject-sam';

describe('PUT /api/settings', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.repositories.coordinators = new InMemoryCoordinatorRepository();
    mocks.repositories.seasons = new InMemorySeasonRepository([
      { endingYear: 2027, startingYear: 2026 },
    ]);
    mocks.repositories.settings = new InMemoryCoordinatorSettingsRepository();
    mocks.repositories.teams = new InMemoryTeamRepository([
      { isOwnTeam: true, name: 'U16-1' },
      { isOwnTeam: true, name: 'U16-2' },
    ]);
  });

  it('makes the chosen primary team the default team of the coordinator', async () => {
    const response = await PUT({
      locals: { coordinatorSession: { subject } },
      request: new Request('http://localhost/api/settings', {
        body: JSON.stringify({ primaryTeamName: 'U16-2', seasonStartingYear: 2026 }),
        headers: { 'content-type': 'application/json' },
        method: 'PUT',
      }),
    } as never);

    expect(response.status).toBe(200);
    await expect(mocks.repositories.coordinators.findBySubject(subject)).resolves.toEqual({
      defaultTeamName: 'U16-2',
      subject,
      teamNames: ['U16-2'],
    });
  });
});
