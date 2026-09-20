export type MembershipRelationship = 'primary' | 'secondary';
export type MembershipStatus = 'active' | 'inactive';
export type ParticipationType = 'trains_and_plays' | 'trains_only';

export interface Membership {
  jerseyNumber?: number;
  participationType: ParticipationType;
  playerAssociationId: string;
  relationship: MembershipRelationship;
  seasonStartingYear: number;
  status: MembershipStatus;
  teamName: string;
}

export interface MembershipRepository {
  findAll(): Promise<Membership[]>;
  find(membership: Membership): Promise<Membership | undefined>;
  save(membership: Membership): Promise<Membership>;
  update(membership: Membership): Promise<Membership>;
}
