import { env } from '$env/dynamic/private';

import type { PlayerRepository } from '../application/players/player-repository';
import type { MembershipRepository } from '../application/memberships/membership-repository';
import type { LocationRepository } from '../application/locations/location-repository';
import type { TrainingSeriesRepository } from '../application/training/training-series-repository';
import type { GameRepository } from '../application/games/game-repository';
import type { SeasonRepository } from '../application/seasons/season-repository';
import type { TeamRepository } from '../application/teams/team-repository';
import { createDatabase } from './persistence/database';
import { readDatabaseUrl } from './persistence/database-configuration';
import { createPostgresPlayerRepository } from './persistence/postgres-player-repository';
import { createPostgresMembershipRepository } from './persistence/postgres-membership-repository';
import { createPostgresLocationRepository } from './persistence/postgres-location-repository';
import { createPostgresTrainingSeriesRepository } from './persistence/postgres-training-series-repository';
import { createPostgresGameRepository } from './persistence/postgres-game-repository';
import { createPostgresSeasonRepository } from './persistence/postgres-season-repository';
import { createPostgresTeamRepository } from './persistence/postgres-team-repository';

type Database = ReturnType<typeof createDatabase>;

let database: Database | undefined;
let players: PlayerRepository | undefined;
let memberships: MembershipRepository | undefined;
let locations: LocationRepository | undefined;
let trainingSeries: TrainingSeriesRepository | undefined;
let games: GameRepository | undefined;
let seasons: SeasonRepository | undefined;
let teams: TeamRepository | undefined;

const currentDatabase = (): Database => {
  database ??= createDatabase(readDatabaseUrl(env));

  return database;
};

export const currentPlayerRepository = (): PlayerRepository => {
  players ??= createPostgresPlayerRepository(currentDatabase());

  return players;
};

export const currentMembershipRepository = (): MembershipRepository => {
  memberships ??= createPostgresMembershipRepository(currentDatabase());

  return memberships;
};

export const currentLocationRepository = (): LocationRepository => {
  locations ??= createPostgresLocationRepository(currentDatabase());

  return locations;
};

export const currentTrainingSeriesRepository = (): TrainingSeriesRepository => {
  trainingSeries ??= createPostgresTrainingSeriesRepository(currentDatabase());

  return trainingSeries;
};

export const currentGameRepository = (): GameRepository => {
  games ??= createPostgresGameRepository(currentDatabase());

  return games;
};

export const currentSeasonRepository = (): SeasonRepository => {
  seasons ??= createPostgresSeasonRepository(currentDatabase());

  return seasons;
};

export const currentTeamRepository = (): TeamRepository => {
  teams ??= createPostgresTeamRepository(currentDatabase());

  return teams;
};
