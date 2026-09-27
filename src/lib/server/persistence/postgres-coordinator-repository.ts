import { and, asc, eq } from 'drizzle-orm';

import type {
  Coordinator,
  CoordinatorProfile,
  CoordinatorRepository,
} from '../../application/coordinators/coordinator-repository';
import { createDatabase } from './database';
import { coordinatorTeams, coordinators, teams } from './schema';

type Database = ReturnType<typeof createDatabase>;

const findTeamId = async (database: Database, teamName: string): Promise<string> => {
  const [team] = await database
    .select({ id: teams.id })
    .from(teams)
    .where(eq(teams.name, teamName))
    .limit(1);

  if (!team) {
    throw new Error(`Team ${teamName} does not exist`);
  }

  return team.id;
};

export const createPostgresCoordinatorRepository = (database: Database): CoordinatorRepository => {
  const findCoordinatorRow = async (subject: string) => {
    const [row] = await database
      .select({
        defaultTeamName: teams.name,
        displayName: coordinators.displayName,
        email: coordinators.email,
        id: coordinators.id,
      })
      .from(coordinators)
      .leftJoin(teams, eq(coordinators.defaultTeamId, teams.id))
      .where(eq(coordinators.subject, subject))
      .limit(1);

    return row;
  };

  const readCoordinator = async (subject: string): Promise<Coordinator | undefined> => {
    const row = await findCoordinatorRow(subject);

    if (!row) {
      return undefined;
    }

    const teamRows = await database
      .select({ name: teams.name })
      .from(coordinatorTeams)
      .innerJoin(teams, eq(coordinatorTeams.teamId, teams.id))
      .where(eq(coordinatorTeams.coordinatorId, row.id))
      .orderBy(asc(teams.name));
    const coordinator: Coordinator = { subject, teamNames: teamRows.map((team) => team.name) };

    if (row.defaultTeamName !== null) {
      coordinator.defaultTeamName = row.defaultTeamName;
    }

    if (row.displayName !== null) {
      coordinator.profile =
        row.email === null
          ? { displayName: row.displayName }
          : { displayName: row.displayName, email: row.email };
    }

    return coordinator;
  };

  const ensureId = async (subject: string): Promise<string> => {
    await database.insert(coordinators).values({ subject }).onConflictDoNothing();
    const row = await findCoordinatorRow(subject);

    if (!row) {
      throw new Error('PostgreSQL did not return the stored coordinator');
    }

    return row.id;
  };

  const readStored = async (subject: string): Promise<Coordinator> => {
    const coordinator = await readCoordinator(subject);

    if (!coordinator) {
      throw new Error('PostgreSQL did not return the stored coordinator');
    }

    return coordinator;
  };

  return {
    findBySubject: readCoordinator,

    async findSubjectsByTeam(teamName: string): Promise<string[]> {
      const rows = await database
        .select({ subject: coordinators.subject })
        .from(coordinatorTeams)
        .innerJoin(coordinators, eq(coordinatorTeams.coordinatorId, coordinators.id))
        .innerJoin(teams, eq(coordinatorTeams.teamId, teams.id))
        .where(eq(teams.name, teamName))
        .orderBy(asc(coordinators.subject));

      return rows.map((row) => row.subject);
    },

    async ensure(subject: string): Promise<Coordinator> {
      await ensureId(subject);

      return readStored(subject);
    },

    async addTeam(subject: string, teamName: string): Promise<Coordinator> {
      const [coordinatorId, teamId] = await Promise.all([
        ensureId(subject),
        findTeamId(database, teamName),
      ]);

      await database
        .insert(coordinatorTeams)
        .values({ coordinatorId, teamId })
        .onConflictDoNothing();

      return readStored(subject);
    },

    async removeTeam(subject: string, teamName: string): Promise<Coordinator> {
      const [coordinatorId, teamId] = await Promise.all([
        ensureId(subject),
        findTeamId(database, teamName),
      ]);

      await database
        .delete(coordinatorTeams)
        .where(
          and(
            eq(coordinatorTeams.coordinatorId, coordinatorId),
            eq(coordinatorTeams.teamId, teamId),
          ),
        );

      return readStored(subject);
    },

    async setDefaultTeam(subject: string, teamName: string | undefined): Promise<Coordinator> {
      const coordinatorId = await ensureId(subject);
      const defaultTeamId = teamName === undefined ? null : await findTeamId(database, teamName);

      await database
        .update(coordinators)
        .set({ defaultTeamId })
        .where(eq(coordinators.id, coordinatorId));

      return readStored(subject);
    },

    async setProfile(subject: string, profile: CoordinatorProfile): Promise<Coordinator> {
      const coordinatorId = await ensureId(subject);

      await database
        .update(coordinators)
        .set({ displayName: profile.displayName, email: profile.email ?? null })
        .where(eq(coordinators.id, coordinatorId));

      return readStored(subject);
    },
  };
};
