import { asc, eq } from 'drizzle-orm';

import type { Team, TeamRepository } from '../../application/teams/team-repository';
import { createDatabase } from './database';
import { teams } from './schema';

type Database = ReturnType<typeof createDatabase>;

const columns = { isOwnTeam: teams.isOwnTeam, name: teams.name };

export const createPostgresTeamRepository = (database: Database): TeamRepository => ({
  async findAll(): Promise<Team[]> {
    return database.select(columns).from(teams).orderBy(asc(teams.name));
  },

  async findAllOwnTeams(): Promise<Team[]> {
    return database
      .select(columns)
      .from(teams)
      .where(eq(teams.isOwnTeam, true))
      .orderBy(asc(teams.name));
  },

  async findByName(name: string): Promise<Team | undefined> {
    const [team] = await database.select(columns).from(teams).where(eq(teams.name, name)).limit(1);

    return team;
  },

  async save(team: Team): Promise<Team> {
    const [storedTeam] = await database.insert(teams).values(team).returning(columns);

    if (!storedTeam) {
      throw new Error('PostgreSQL did not return the stored team');
    }

    return storedTeam;
  },

  async updateName(currentName: string, name: string): Promise<Team> {
    const [updatedTeam] = await database
      .update(teams)
      .set({ name })
      .where(eq(teams.name, currentName))
      .returning(columns);

    if (!updatedTeam) {
      throw new Error('Team does not exist');
    }

    return updatedTeam;
  },

  async deleteByName(name: string): Promise<void> {
    await database.delete(teams).where(eq(teams.name, name));
  },
});
