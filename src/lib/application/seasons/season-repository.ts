export interface Season {
  endingYear: number;
  startingYear: number;
}

export interface SeasonRepository {
  findAll(): Promise<Season[]>;
  findByStartingYear(startingYear: number): Promise<Season | undefined>;
  save(season: Season): Promise<Season>;
  updateStartingYear(currentStartingYear: number, startingYear: number): Promise<Season>;
  deleteByStartingYear(startingYear: number): Promise<void>;
}
