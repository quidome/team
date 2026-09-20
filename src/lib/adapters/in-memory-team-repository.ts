import type { Team, TeamRepository } from '../application/teams/team-repository';

export class InMemoryTeamRepository implements TeamRepository {
  private readonly teams = new Map<string, Team>();

  constructor(initialTeams: Team[] = []) {
    for (const team of initialTeams) {
      this.teams.set(team.name, team);
    }
  }

  async findAll(): Promise<Team[]> {
    return [...this.teams.values()].sort((left, right) => left.name.localeCompare(right.name));
  }

  async findByName(name: string): Promise<Team | undefined> {
    return this.teams.get(name);
  }

  async save(team: Team): Promise<Team> {
    this.teams.set(team.name, team);

    return team;
  }

  async updateName(currentName: string, name: string): Promise<Team> {
    if (!this.teams.has(currentName)) {
      throw new Error('Team does not exist');
    }

    const updated = { name };
    this.teams.delete(currentName);
    this.teams.set(name, updated);

    return updated;
  }

  async deleteByName(name: string): Promise<void> {
    this.teams.delete(name);
  }
}
