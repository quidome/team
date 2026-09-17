import { env } from '$env/dynamic/private';

import type { SeasonRepository } from '../application/seasons/season-repository';
import type { TeamRepository } from '../application/teams/team-repository';
import { createDatabase } from './persistence/database';
import { readDatabaseUrl } from './persistence/database-configuration';
import { createPostgresSeasonRepository } from './persistence/postgres-season-repository';
import { createPostgresTeamRepository } from './persistence/postgres-team-repository';

type Database = ReturnType<typeof createDatabase>;

let database: Database | undefined;
let seasons: SeasonRepository | undefined;
let teams: TeamRepository | undefined;

const currentDatabase = (): Database => {
  database ??= createDatabase(readDatabaseUrl(env));

  return database;
};

export const currentSeasonRepository = (): SeasonRepository => {
  seasons ??= createPostgresSeasonRepository(currentDatabase());

  return seasons;
};

export const currentTeamRepository = (): TeamRepository => {
  teams ??= createPostgresTeamRepository(currentDatabase());

  return teams;
};
