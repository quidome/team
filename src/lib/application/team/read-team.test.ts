import { describe, expect, it } from 'vitest';

import { InMemoryMembershipRepository } from '../../adapters/in-memory-membership-repository';
import { InMemoryPlayerRepository } from '../../adapters/in-memory-player-repository';
import { readTeam } from './read-team';

describe('read team', () => {
  it('joins current-team memberships and calculates normal age groups', async () => {
    const players = new InMemoryPlayerRepository([
      { associationId: 'avery', birthDate: '2011-06-15', name: 'Avery' },
      { associationId: 'blake', birthDate: '2010-02-10', name: 'Blake' },
    ]);
    const memberships = new InMemoryMembershipRepository([
      {
        jerseyNumber: 7,
        participationType: 'trains_and_plays',
        playerAssociationId: 'avery',
        relationship: 'primary',
        seasonStartingYear: 2026,
        status: 'active',
        teamName: 'U16-1',
      },
      {
        participationType: 'trains_only',
        playerAssociationId: 'blake',
        relationship: 'secondary',
        seasonStartingYear: 2026,
        status: 'active',
        teamName: 'U18-1',
      },
    ]);

    await expect(
      readTeam(players, memberships, { seasonStartingYear: 2026, teamName: 'U16-1' }),
    ).resolves.toEqual([
      {
        associationId: 'avery',
        birthDate: '2011-06-15',
        membership: {
          jerseyNumber: 7,
          participationType: 'trains_and_plays',
          playerAssociationId: 'avery',
          relationship: 'primary',
          seasonStartingYear: 2026,
          status: 'active',
          teamName: 'U16-1',
        },
        name: 'Avery',
        normalAgeGroup: 'U16',
      },
      {
        associationId: 'blake',
        birthDate: '2010-02-10',
        name: 'Blake',
        normalAgeGroup: 'U18',
      },
    ]);
  });
});
