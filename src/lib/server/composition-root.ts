import { env } from '$env/dynamic/private';

import type { SeasonRepository } from '../application/seasons/season-repository';
import { createDatabase } from './persistence/database';
import { readDatabaseUrl } from './persistence/database-configuration';
import { createPostgresSeasonRepository } from './persistence/postgres-season-repository';

let seasons: SeasonRepository | undefined;

export const currentSeasonRepository = (): SeasonRepository => {
  seasons ??= createPostgresSeasonRepository(createDatabase(readDatabaseUrl(env)));

  return seasons;
};
