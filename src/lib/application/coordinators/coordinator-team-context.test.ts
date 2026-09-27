import { describe, expect, it } from 'vitest';

import { InMemoryCoordinatorRepository } from '../../adapters/in-memory-coordinator-repository';
import { InMemoryCoordinatorSettingsRepository } from '../../adapters/in-memory-coordinator-settings-repository';
import { resolveCoordinatorTeamName } from './coordinator-team-context';

const sam = 'pocket-id-subject-sam';

describe('coordinator team context', () => {
  it('Opening the default team after login', async () => {
    const coordinators = new InMemoryCoordinatorRepository([
      { defaultTeamName: 'U16-2', subject: sam, teamNames: ['U16-1', 'U16-2'] },
    ]);
    const settings = new InMemoryCoordinatorSettingsRepository({
      primaryTeamName: 'U16-1',
      seasonStartingYear: 2026,
    });

    await expect(resolveCoordinatorTeamName(coordinators, settings, sam)).resolves.toBe('U16-2');
  });

  it('takes over the installation primary team for a coordinator without teams', async () => {
    const coordinators = new InMemoryCoordinatorRepository();
    const settings = new InMemoryCoordinatorSettingsRepository({
      primaryTeamName: 'U16-1',
      seasonStartingYear: 2026,
    });

    await expect(resolveCoordinatorTeamName(coordinators, settings, sam)).resolves.toBe('U16-1');
    await expect(coordinators.findBySubject(sam)).resolves.toEqual({
      defaultTeamName: 'U16-1',
      subject: sam,
      teamNames: ['U16-1'],
    });
  });

  it('does not take over the primary team when the coordinator already has teams', async () => {
    const coordinators = new InMemoryCoordinatorRepository([
      { subject: sam, teamNames: ['U16-2'] },
    ]);
    const settings = new InMemoryCoordinatorSettingsRepository({
      primaryTeamName: 'U16-1',
      seasonStartingYear: 2026,
    });

    await expect(resolveCoordinatorTeamName(coordinators, settings, sam)).resolves.toBeUndefined();
  });

  it('returns no team when nothing is configured', async () => {
    await expect(
      resolveCoordinatorTeamName(
        new InMemoryCoordinatorRepository(),
        new InMemoryCoordinatorSettingsRepository(),
        sam,
      ),
    ).resolves.toBeUndefined();
  });
});
