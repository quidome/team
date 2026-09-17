import { integer, pgTable, timestamp, uniqueIndex, uuid } from 'drizzle-orm/pg-core';

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
