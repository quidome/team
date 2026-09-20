import { asc, eq } from 'drizzle-orm';

import type { Team, TeamRepository } from '../../application/teams/team-repository';
import { createDatabase } from './database';
import { teams } from './schema';

type Database = ReturnType<typeof createDatabase>;

export const createPostgresTeamRepository = (database: Database): TeamRepository => ({
  async findAll(): Promise<Team[]> {
    return database.select({ name: teams.name }).from(teams).orderBy(asc(teams.name));
  },

  async findByName(name: string): Promise<Team | undefined> {
    const [team] = await database
      .select({ name: teams.name })
      .from(teams)
      .where(eq(teams.name, name))
      .limit(1);

    return team;
  },

  async save(team: Team): Promise<Team> {
    const [storedTeam] = await database.insert(teams).values(team).returning({ name: teams.name });

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
      .returning({ name: teams.name });

    if (!updatedTeam) {
      throw new Error('Team does not exist');
    }

    return updatedTeam;
  },

  async deleteByName(name: string): Promise<void> {
    await database.delete(teams).where(eq(teams.name, name));
  },
});
