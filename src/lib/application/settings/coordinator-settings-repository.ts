export interface CoordinatorSettings {
  primaryTeamName: string;
  seasonStartingYear: number;
}

export interface CoordinatorSettingsRepository {
  get(): Promise<CoordinatorSettings | undefined>;
  save(settings: CoordinatorSettings): Promise<CoordinatorSettings>;
}
