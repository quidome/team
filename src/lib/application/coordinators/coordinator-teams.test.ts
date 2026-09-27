import { describe, expect, it } from 'vitest';

import { InMemoryCoordinatorRepository } from '../../adapters/in-memory-coordinator-repository';
import { InMemoryTeamRepository } from '../../adapters/in-memory-team-repository';
import {
  CoordinatorError,
  giveUpTeamResponsibility,
  recordCoordinatorLogin,
  setCoordinatorProfile,
  setDefaultTeam,
  takeTeamResponsibility,
} from './coordinator-teams';

const sam = 'pocket-id-subject-sam';
const robin = 'pocket-id-subject-robin';

const createTeams = () =>
  new InMemoryTeamRepository([
    { isOwnTeam: true, name: 'U16-1' },
    { isOwnTeam: true, name: 'U16-2' },
    { isOwnTeam: true, name: 'U18-1' },
  ]);

describe('coordinator teams', () => {
  it('Recording a coordinator at first login', async () => {
    const coordinators = new InMemoryCoordinatorRepository();

    const coordinator = await recordCoordinatorLogin(coordinators, sam);

    expect(coordinator).toEqual({ subject: sam, teamNames: [] });
    await expect(coordinators.findBySubject(sam)).resolves.toEqual(coordinator);
  });

  it('Recognising a returning coordinator', async () => {
    const coordinators = new InMemoryCoordinatorRepository([
      { defaultTeamName: 'U16-1', subject: sam, teamNames: ['U16-1'] },
    ]);

    const coordinator = await recordCoordinatorLogin(coordinators, sam);

    expect(coordinator).toEqual({ defaultTeamName: 'U16-1', subject: sam, teamNames: ['U16-1'] });
    await expect(coordinators.findSubjectsByTeam('U16-1')).resolves.toEqual([sam]);
  });

  it('Taking responsibility for more than one team', async () => {
    const coordinators = new InMemoryCoordinatorRepository();
    const teams = createTeams();

    await takeTeamResponsibility(coordinators, teams, sam, 'U16-1');
    const coordinator = await takeTeamResponsibility(coordinators, teams, sam, 'U16-2');

    expect(coordinator.teamNames).toEqual(['U16-1', 'U16-2']);
  });

  it('rejects responsibility for a team that does not exist', async () => {
    const coordinators = new InMemoryCoordinatorRepository();

    await expect(
      takeTeamResponsibility(coordinators, createTeams(), sam, 'U20-1'),
    ).rejects.toBeInstanceOf(CoordinatorError);
  });

  it('Sharing a team between coordinators', async () => {
    const coordinators = new InMemoryCoordinatorRepository();
    const teams = createTeams();

    await takeTeamResponsibility(coordinators, teams, sam, 'U16-1');
    await takeTeamResponsibility(coordinators, teams, robin, 'U16-1');

    await expect(coordinators.findSubjectsByTeam('U16-1')).resolves.toEqual([robin, sam]);
  });

  it('Setting the default team', async () => {
    const coordinators = new InMemoryCoordinatorRepository([
      { subject: sam, teamNames: ['U16-1', 'U16-2'] },
    ]);

    const coordinator = await setDefaultTeam(coordinators, sam, 'U16-1');

    expect(coordinator.defaultTeamName).toBe('U16-1');
  });

  it("Rejecting a default team outside the coordinator's teams", async () => {
    const coordinators = new InMemoryCoordinatorRepository([
      { subject: sam, teamNames: ['U16-1'] },
    ]);

    await expect(setDefaultTeam(coordinators, sam, 'U18-1')).rejects.toThrow(
      'Coordinator is not responsible for U18-1',
    );
    await expect(coordinators.findBySubject(sam)).resolves.toEqual({
      subject: sam,
      teamNames: ['U16-1'],
    });
  });

  it('Giving up responsibility for the default team', async () => {
    const coordinators = new InMemoryCoordinatorRepository([
      { defaultTeamName: 'U16-1', subject: sam, teamNames: ['U16-1', 'U16-2'] },
    ]);

    const coordinator = await giveUpTeamResponsibility(coordinators, sam, 'U16-1');

    expect(coordinator).toEqual({ subject: sam, teamNames: ['U16-2'] });
  });

  it('keeps the default team when giving up another team', async () => {
    const coordinators = new InMemoryCoordinatorRepository([
      { defaultTeamName: 'U16-1', subject: sam, teamNames: ['U16-1', 'U16-2'] },
    ]);

    const coordinator = await giveUpTeamResponsibility(coordinators, sam, 'U16-2');

    expect(coordinator).toEqual({ defaultTeamName: 'U16-1', subject: sam, teamNames: ['U16-1'] });
  });

  it('Keeping a coordinator profile', async () => {
    const coordinators = new InMemoryCoordinatorRepository([{ subject: sam, teamNames: [] }]);

    const coordinator = await setCoordinatorProfile(coordinators, sam, {
      displayName: ' Sam Jansen ',
      email: 'sam@example.test',
    });

    expect(coordinator.profile).toEqual({ displayName: 'Sam Jansen', email: 'sam@example.test' });
    await expect(coordinators.findBySubject(sam)).resolves.toEqual(coordinator);
  });

  it('rejects a profile without a display name or with an invalid email', async () => {
    const coordinators = new InMemoryCoordinatorRepository();

    await expect(setCoordinatorProfile(coordinators, sam, { displayName: '  ' })).rejects.toThrow(
      'Display name is required',
    );
    await expect(
      setCoordinatorProfile(coordinators, sam, { displayName: 'Sam', email: 'not-an-email' }),
    ).rejects.toThrow('Email address is not valid');
  });
});
