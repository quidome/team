import { describe, expect, it } from 'vitest';

import { InMemoryMembershipRepository } from '../../adapters/in-memory-membership-repository';
import { InMemoryPlayerRepository } from '../../adapters/in-memory-player-repository';
import { readTeam } from './read-team';

describe('read team', () => {
  it('joins current-team memberships and calculates normal age groups', async () => {
    const players = new InMemoryPlayerRepository([
      { associationId: 'avery', birthDate: '2011-06-15', firstName: 'Avery', id: 'avery-id' },
      { associationId: 'blake', birthDate: '2010-02-10', firstName: 'Blake', id: 'blake-id' },
    ]);
    const memberships = new InMemoryMembershipRepository([
      {
        jerseyNumber: 7,
        participationType: 'trains_and_plays',
        playerId: 'avery-id',
        relationship: 'primary',
        seasonStartingYear: 2026,
        status: 'active',
        teamName: 'U16-1',
      },
      {
        participationType: 'trains_only',
        playerId: 'blake-id',
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
        firstName: 'Avery',
        id: 'avery-id',
        membership: {
          jerseyNumber: 7,
          participationType: 'trains_and_plays',
          playerId: 'avery-id',
          relationship: 'primary',
          seasonStartingYear: 2026,
          status: 'active',
          teamName: 'U16-1',
        },
        normalAgeGroup: 'U16',
      },
      {
        associationId: 'blake',
        birthDate: '2010-02-10',
        firstName: 'Blake',
        id: 'blake-id',
        normalAgeGroup: 'U18',
      },
    ]);
  });

  it('omits the normal age group for a player with no recorded birthdate', async () => {
    const players = new InMemoryPlayerRepository([
      { associationId: 'avery', birthDate: '2011-06-15', firstName: 'Avery', id: 'avery-id' },
      { firstName: 'Casey', id: 'casey-id' },
    ]);
    const memberships = new InMemoryMembershipRepository();

    await expect(
      readTeam(players, memberships, { seasonStartingYear: 2026, teamName: 'U16-1' }),
    ).resolves.toEqual([
      {
        associationId: 'avery',
        birthDate: '2011-06-15',
        firstName: 'Avery',
        id: 'avery-id',
        normalAgeGroup: 'U16',
      },
      {
        firstName: 'Casey',
        id: 'casey-id',
      },
    ]);
  });
});
