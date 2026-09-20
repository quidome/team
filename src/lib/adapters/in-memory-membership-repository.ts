import type {
  Membership,
  MembershipRepository,
} from '../application/memberships/membership-repository';

const membershipKey = (membership: Membership) =>
  [membership.playerAssociationId, membership.teamName, membership.seasonStartingYear].join('|');

export class InMemoryMembershipRepository implements MembershipRepository {
  private readonly memberships = new Map<string, Membership>();

  constructor(initialMemberships: Membership[] = []) {
    for (const membership of initialMemberships) {
      this.memberships.set(membershipKey(membership), membership);
    }
  }

  async findAll(): Promise<Membership[]> {
    return [...this.memberships.values()];
  }

  async find(membership: Membership): Promise<Membership | undefined> {
    return this.memberships.get(membershipKey(membership));
  }

  async save(membership: Membership): Promise<Membership> {
    this.memberships.set(membershipKey(membership), membership);

    return membership;
  }

  async update(membership: Membership): Promise<Membership> {
    const key = membershipKey(membership);

    if (!this.memberships.has(key)) {
      throw new Error('Membership does not exist');
    }

    this.memberships.set(key, membership);

    return membership;
  }
}
