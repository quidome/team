import { desc, eq } from 'drizzle-orm';

import type { Season, SeasonRepository } from '../../application/seasons/season-repository';
import { createDatabase } from './database';
import { seasons } from './schema';

type Database = ReturnType<typeof createDatabase>;

export const createPostgresSeasonRepository = (database: Database): SeasonRepository => ({
  async findAll(): Promise<Season[]> {
    return database
      .select({
        endingYear: seasons.endingYear,
        startingYear: seasons.startingYear,
      })
      .from(seasons)
      .orderBy(desc(seasons.startingYear));
  },

  async findByStartingYear(startingYear: number): Promise<Season | undefined> {
    const [season] = await database
      .select({
        endingYear: seasons.endingYear,
        startingYear: seasons.startingYear,
      })
      .from(seasons)
      .where(eq(seasons.startingYear, startingYear))
      .limit(1);

    return season;
  },

  async save(season: Season): Promise<Season> {
    const [storedSeason] = await database.insert(seasons).values(season).returning({
      endingYear: seasons.endingYear,
      startingYear: seasons.startingYear,
    });

    if (!storedSeason) {
      throw new Error('PostgreSQL did not return the stored season');
    }

    return storedSeason;
  },

  async updateStartingYear(currentStartingYear: number, startingYear: number): Promise<Season> {
    const [updatedSeason] = await database
      .update(seasons)
      .set({ endingYear: startingYear + 1, startingYear })
      .where(eq(seasons.startingYear, currentStartingYear))
      .returning({ endingYear: seasons.endingYear, startingYear: seasons.startingYear });

    if (!updatedSeason) {
      throw new Error('Season does not exist');
    }

    return updatedSeason;
  },

  async deleteByStartingYear(startingYear: number): Promise<void> {
    await database.delete(seasons).where(eq(seasons.startingYear, startingYear));
  },
});
