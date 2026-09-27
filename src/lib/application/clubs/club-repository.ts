export interface Club {
  isOwnClub: boolean;
  name: string;
  sourceClubId?: number;
}

export interface ClubRepository {
  findAll(): Promise<Club[]>;
  findByName(name: string): Promise<Club | undefined>;
  findBySourceClubId(sourceClubId: number): Promise<Club | undefined>;
  findOwnClub(): Promise<Club | undefined>;
  findTeamNames(clubName: string): Promise<string[]>;
  save(club: Club): Promise<Club>;
  assignTeam(clubName: string, teamName: string): Promise<void>;
}
