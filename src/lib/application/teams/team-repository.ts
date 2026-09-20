export interface Team {
  name: string;
}

export interface TeamRepository {
  findAll(): Promise<Team[]>;
  findByName(name: string): Promise<Team | undefined>;
  save(team: Team): Promise<Team>;
  updateName(currentName: string, name: string): Promise<Team>;
  deleteByName(name: string): Promise<void>;
}
