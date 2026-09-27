import type { Club, ClubRepository } from '../application/clubs/club-repository';

export class InMemoryClubRepository implements ClubRepository {
  private readonly clubs = new Map<string, Club>();
  private readonly teamClubs = new Map<string, string>();

  constructor(initialClubs: Club[] = []) {
    for (const club of initialClubs) {
      this.clubs.set(club.name, club);
    }
  }

  async findAll(): Promise<Club[]> {
    return [...this.clubs.values()].sort((left, right) => left.name.localeCompare(right.name));
  }

  async findByName(name: string): Promise<Club | undefined> {
    return this.clubs.get(name);
  }

  async findBySourceClubId(sourceClubId: number): Promise<Club | undefined> {
    return [...this.clubs.values()].find((club) => club.sourceClubId === sourceClubId);
  }

  async findOwnClub(): Promise<Club | undefined> {
    return [...this.clubs.values()].find((club) => club.isOwnClub);
  }

  async findTeamNames(clubName: string): Promise<string[]> {
    return [...this.teamClubs.entries()]
      .filter(([, club]) => club === clubName)
      .map(([team]) => team)
      .sort((left, right) => left.localeCompare(right));
  }

  async save(club: Club): Promise<Club> {
    this.clubs.set(club.name, club);

    return club;
  }

  async assignTeam(clubName: string, teamName: string): Promise<void> {
    this.teamClubs.set(teamName, clubName);
  }
}
