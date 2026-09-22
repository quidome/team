import { describe, expect, it } from 'vitest';

import { InMemoryTeamRepository } from '../../adapters/in-memory-team-repository';
import { configureTeam } from './configure-team';

describe('configure team', () => {
  it('stores a team by name', async () => {
    const teams = new InMemoryTeamRepository();

    const team = await configureTeam(teams, 'U16-1');

    expect(team).toEqual({ isOwnTeam: true, name: 'U16-1' });
    await expect(teams.findByName('U16-1')).resolves.toEqual(team);
  });

  it('returns an already-configured team rather than creating a duplicate', async () => {
    const teams = new InMemoryTeamRepository([{ isOwnTeam: true, name: 'U16-1' }]);

    await expect(configureTeam(teams, 'U16-1')).resolves.toEqual({
      isOwnTeam: true,
      name: 'U16-1',
    });
  });
});
