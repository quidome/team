import { asc, eq } from 'drizzle-orm';

import type { Club, ClubRepository } from '../../application/clubs/club-repository';
import { createDatabase } from './database';
import { clubs, teams } from './schema';

type Database = ReturnType<typeof createDatabase>;

const columns = {
  isOwnClub: clubs.isOwnClub,
  name: clubs.name,
  sourceClubId: clubs.sourceClubId,
};

const toClub = (row: { isOwnClub: boolean; name: string; sourceClubId: number | null }): Club =>
  row.sourceClubId === null
    ? { isOwnClub: row.isOwnClub, name: row.name }
    : { isOwnClub: row.isOwnClub, name: row.name, sourceClubId: row.sourceClubId };

export const createPostgresClubRepository = (database: Database): ClubRepository => ({
  async findAll(): Promise<Club[]> {
    const rows = await database.select(columns).from(clubs).orderBy(asc(clubs.name));

    return rows.map(toClub);
  },

  async findByName(name: string): Promise<Club | undefined> {
    const [row] = await database.select(columns).from(clubs).where(eq(clubs.name, name)).limit(1);

    return row ? toClub(row) : undefined;
  },

  async findBySourceClubId(sourceClubId: number): Promise<Club | undefined> {
    const [row] = await database
      .select(columns)
      .from(clubs)
      .where(eq(clubs.sourceClubId, sourceClubId))
      .limit(1);

    return row ? toClub(row) : undefined;
  },

  async findOwnClub(): Promise<Club | undefined> {
    const [row] = await database
      .select(columns)
      .from(clubs)
      .where(eq(clubs.isOwnClub, true))
      .limit(1);

    return row ? toClub(row) : undefined;
  },

  async findTeamNames(clubName: string): Promise<string[]> {
    const rows = await database
      .select({ name: teams.name })
      .from(teams)
      .innerJoin(clubs, eq(teams.clubId, clubs.id))
      .where(eq(clubs.name, clubName))
      .orderBy(asc(teams.name));

    return rows.map((row) => row.name);
  },

  async save(club: Club): Promise<Club> {
    const values = {
      isOwnClub: club.isOwnClub,
      name: club.name,
      sourceClubId: club.sourceClubId ?? null,
    };
    const [row] = await database
      .insert(clubs)
      .values(values)
      .onConflictDoUpdate({
        set: { isOwnClub: values.isOwnClub, sourceClubId: values.sourceClubId },
        target: clubs.name,
      })
      .returning(columns);

    if (!row) {
      throw new Error('PostgreSQL did not return the stored club');
    }

    return toClub(row);
  },

  async assignTeam(clubName: string, teamName: string): Promise<void> {
    const [club] = await database
      .select({ id: clubs.id })
      .from(clubs)
      .where(eq(clubs.name, clubName))
      .limit(1);

    if (!club) {
      throw new Error(`Club ${clubName} does not exist`);
    }

    const updated = await database
      .update(teams)
      .set({ clubId: club.id })
      .where(eq(teams.name, teamName))
      .returning({ id: teams.id });

    if (updated.length === 0) {
      throw new Error(`Team ${teamName} does not exist`);
    }
  },
});
