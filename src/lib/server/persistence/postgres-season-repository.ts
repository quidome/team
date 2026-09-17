import { eq } from 'drizzle-orm';

import type { Season, SeasonRepository } from '../../application/seasons/season-repository';
import { createDatabase } from './database';
import { seasons } from './schema';

type Database = ReturnType<typeof createDatabase>;

export const createPostgresSeasonRepository = (database: Database): SeasonRepository => ({
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
});
