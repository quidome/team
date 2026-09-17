import type { Team, TeamRepository } from '../application/teams/team-repository';

export class InMemoryTeamRepository implements TeamRepository {
  private readonly teams = new Map<string, Team>();

  constructor(initialTeams: Team[] = []) {
    for (const team of initialTeams) {
      this.teams.set(team.name, team);
    }
  }

  async findByName(name: string): Promise<Team | undefined> {
    return this.teams.get(name);
  }

  async save(team: Team): Promise<Team> {
    this.teams.set(team.name, team);

    return team;
  }
}
