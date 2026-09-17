import { describe, expect, it } from 'vitest';

import { InMemoryMembershipRepository } from '../../adapters/in-memory-membership-repository';
import { configureMembership } from './configure-membership';

describe('configure season-specific team membership', () => {
  it('stores membership status, jersey number, participation type, and relationship', async () => {
    const memberships = new InMemoryMembershipRepository();
    const membership = {
      jerseyNumber: 7,
      participationType: 'trains_and_plays' as const,
      playerAssociationId: '12345',
      relationship: 'primary' as const,
      seasonStartingYear: 2026,
      status: 'active' as const,
      teamName: 'U16-1',
    };

    await expect(configureMembership(memberships, membership)).resolves.toEqual(membership);
    await expect(memberships.find(membership)).resolves.toEqual(membership);
  });

  it('returns an existing membership rather than creating a duplicate', async () => {
    const membership = {
      jerseyNumber: 7,
      participationType: 'trains_and_plays' as const,
      playerAssociationId: '12345',
      relationship: 'primary' as const,
      seasonStartingYear: 2026,
      status: 'active' as const,
      teamName: 'U16-1',
    };
    const memberships = new InMemoryMembershipRepository([membership]);

    await expect(configureMembership(memberships, membership)).resolves.toEqual(membership);
  });
});
