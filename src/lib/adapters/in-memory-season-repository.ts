import type { Season, SeasonRepository } from '../application/seasons/season-repository';

export class InMemorySeasonRepository implements SeasonRepository {
  private readonly seasons = new Map<number, Season>();

  constructor(initialSeasons: Season[] = []) {
    for (const season of initialSeasons) {
      this.seasons.set(season.startingYear, season);
    }
  }

  async findAll(): Promise<Season[]> {
    return [...this.seasons.values()].sort((left, right) => right.startingYear - left.startingYear);
  }

  async findByStartingYear(startingYear: number): Promise<Season | undefined> {
    return this.seasons.get(startingYear);
  }

  async save(season: Season): Promise<Season> {
    this.seasons.set(season.startingYear, season);

    return season;
  }

  async updateStartingYear(currentStartingYear: number, startingYear: number): Promise<Season> {
    const existing = this.seasons.get(currentStartingYear);

    if (!existing) {
      throw new Error('Season does not exist');
    }

    const updated = { endingYear: startingYear + 1, startingYear };
    this.seasons.delete(currentStartingYear);
    this.seasons.set(startingYear, updated);

    return updated;
  }

  async deleteByStartingYear(startingYear: number): Promise<void> {
    this.seasons.delete(startingYear);
  }
}
