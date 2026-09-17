import {
  date,
  integer,
  pgEnum,
  pgTable,
  text,
  timestamp,
  uniqueIndex,
  uuid,
} from 'drizzle-orm/pg-core';

export const players = pgTable(
  'players',
  {
    id: uuid('id').defaultRandom().primaryKey(),
    associationId: text('association_id').notNull(),
    birthDate: date('birth_date', { mode: 'string' }).notNull(),
    name: text('name').notNull(),
    createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
  },
  (table) => [uniqueIndex('players_association_id_unique').on(table.associationId)],
);

export const teams = pgTable(
  'teams',
  {
    id: uuid('id').defaultRandom().primaryKey(),
    name: text('name').notNull(),
    createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
  },
  (table) => [uniqueIndex('teams_name_unique').on(table.name)],
);

export const seasons = pgTable(
  'seasons',
  {
    id: uuid('id').defaultRandom().primaryKey(),
    endingYear: integer('ending_year').notNull(),
    startingYear: integer('starting_year').notNull(),
    createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
  },
  (table) => [uniqueIndex('seasons_starting_year_unique').on(table.startingYear)],
);
