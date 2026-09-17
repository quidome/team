import type { Membership, MembershipRepository } from './membership-repository';

export const configureMembership = async (
  memberships: MembershipRepository,
  membership: Membership,
): Promise<Membership> => {
  const existingMembership = await memberships.find(membership);

  if (existingMembership) {
    return existingMembership;
  }

  return memberships.save(membership);
};
