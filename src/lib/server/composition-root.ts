import { sql } from 'drizzle-orm';

import { env } from '$env/dynamic/private';

import type { PlayerRepository } from '../application/players/player-repository';
import type { MembershipRepository } from '../application/memberships/membership-repository';
import type { LocationRepository } from '../application/locations/location-repository';
import type { TrainingSeriesRepository } from '../application/training/training-series-repository';
import type { GameRepository } from '../application/games/game-repository';
import type { ParticipationRepository } from '../application/participation/participation-repository';
import type { DutyRepository } from '../application/duties/duty-repository';
import type { TaskRepository } from '../application/tasks/task-repository';
import type { SeasonRepository } from '../application/seasons/season-repository';
import type { TeamRepository } from '../application/teams/team-repository';
import type { GameImportRepository } from '../application/imports/game-import-repository';
import type { AuditRepository } from '../application/audit/audit-repository';
import type { CoordinatorSettingsRepository } from '../application/settings/coordinator-settings-repository';
import { createDatabase, type Database } from './persistence/database';
import { readDatabaseUrl } from './persistence/database-configuration';
import { createPostgresPlayerRepository } from './persistence/postgres-player-repository';
import { createPostgresMembershipRepository } from './persistence/postgres-membership-repository';
import { createPostgresLocationRepository } from './persistence/postgres-location-repository';
import { createPostgresTrainingSeriesRepository } from './persistence/postgres-training-series-repository';
import { createPostgresGameRepository } from './persistence/postgres-game-repository';
import { createPostgresParticipationRepository } from './persistence/postgres-participation-repository';
import { createPostgresDutyRepository } from './persistence/postgres-duty-repository';
import { createPostgresTaskRepository } from './persistence/postgres-task-repository';
import { createPostgresSeasonRepository } from './persistence/postgres-season-repository';
import { createPostgresTeamRepository } from './persistence/postgres-team-repository';
import { createPostgresGameImportRepository } from './persistence/postgres-game-import-repository';
import { createPostgresAuditRepository } from './persistence/postgres-audit-repository';
import { createPostgresCoordinatorSettingsRepository } from './persistence/postgres-coordinator-settings-repository';

let database: Database | undefined;
let players: PlayerRepository | undefined;
let memberships: MembershipRepository | undefined;
let locations: LocationRepository | undefined;
let trainingSeries: TrainingSeriesRepository | undefined;
let games: GameRepository | undefined;
let participation: ParticipationRepository | undefined;
let duties: DutyRepository | undefined;
let tasks: TaskRepository | undefined;
let seasons: SeasonRepository | undefined;
let teams: TeamRepository | undefined;
let gameImports: GameImportRepository | undefined;
let audit: AuditRepository | undefined;
let coordinatorSettings: CoordinatorSettingsRepository | undefined;
let shutdownRegistered = false;

const currentDatabase = (): Database => {
  database ??= createDatabase(readDatabaseUrl(env));

  return database;
};

export const closeCurrentDatabase = async (): Promise<void> => {
  const activeDatabase = database;
  database = undefined;
  players = undefined;
  memberships = undefined;
  locations = undefined;
  trainingSeries = undefined;
  games = undefined;
  participation = undefined;
  duties = undefined;
  tasks = undefined;
  seasons = undefined;
  teams = undefined;
  gameImports = undefined;
  audit = undefined;
  coordinatorSettings = undefined;

  if (activeDatabase) {
    await activeDatabase.close();
  }
};

export const registerDatabaseShutdown = (): void => {
  if (shutdownRegistered) {
    return;
  }

  shutdownRegistered = true;
  const shutdown = () => {
    void closeCurrentDatabase()
      .then(() => process.exit(0))
      .catch((error: unknown) => {
        console.error('Failed to close the database during shutdown.', error);
        process.exit(1);
      });
  };

  process.once('SIGINT', shutdown);
  process.once('SIGTERM', shutdown);
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

export const currentParticipationRepository = (): ParticipationRepository => {
  participation ??= createPostgresParticipationRepository(currentDatabase());

  return participation;
};

export const currentDutyRepository = (): DutyRepository => {
  duties ??= createPostgresDutyRepository(currentDatabase());

  return duties;
};

export const currentTaskRepository = (): TaskRepository => {
  tasks ??= createPostgresTaskRepository(currentDatabase());

  return tasks;
};

export const currentSeasonRepository = (): SeasonRepository => {
  seasons ??= createPostgresSeasonRepository(currentDatabase());

  return seasons;
};

export const currentTeamRepository = (): TeamRepository => {
  teams ??= createPostgresTeamRepository(currentDatabase());

  return teams;
};

export const currentGameImportRepository = (): GameImportRepository => {
  gameImports ??= createPostgresGameImportRepository(currentDatabase());

  return gameImports;
};

export const currentAuditRepository = (): AuditRepository => {
  audit ??= createPostgresAuditRepository(currentDatabase());

  return audit;
};

export const currentCoordinatorSettingsRepository = (): CoordinatorSettingsRepository => {
  coordinatorSettings ??= createPostgresCoordinatorSettingsRepository(currentDatabase());

  return coordinatorSettings;
};

export const withCurrentImportTransaction = async <T>(
  work: (repositories: {
    audit: AuditRepository;
    duties: DutyRepository;
    gameImports: GameImportRepository;
    games: GameRepository;
  }) => Promise<T>,
): Promise<T> =>
  currentDatabase().transaction(async (transaction) =>
    work({
      audit: createPostgresAuditRepository(transaction),
      duties: createPostgresDutyRepository(transaction),
      gameImports: createPostgresGameImportRepository(transaction),
      games: createPostgresGameRepository(transaction),
    }),
  );

export const checkDatabaseConnection = async (): Promise<boolean> => {
  try {
    await currentDatabase().execute(sql`select 1`);
    return true;
  } catch {
    return false;
  }
};
