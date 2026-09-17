export interface Team {
  name: string;
}

export interface TeamRepository {
  findByName(name: string): Promise<Team | undefined>;
  save(team: Team): Promise<Team>;
}
