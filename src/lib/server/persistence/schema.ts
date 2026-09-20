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

export const membershipRelationship = pgEnum('membership_relationship', ['primary', 'secondary']);
export const membershipStatus = pgEnum('membership_status', ['active', 'inactive']);
export const participationType = pgEnum('participation_type', ['trains_and_plays', 'trains_only']);

export const gameOccurrenceStatus = pgEnum('game_occurrence_status', ['cancelled', 'scheduled']);

export const absenceReason = pgEnum('absence_reason', ['illness', 'injury', 'other']);
export const participationOccurrenceType = pgEnum('participation_occurrence_type', [
  'game',
  'training',
]);
export const participationStatus = pgEnum('participation_status', ['absent', 'present']);
export const dutyType = pgEnum('duty_type', ['driving', 'jury', 'referee']);
export const dutySlotStatus = pgEnum('duty_slot_status', [
  'assigned',
  'cancelled',
  'completed',
  'incomplete',
  'open',
]);
export const dutySignupStatus = pgEnum('duty_signup_status', [
  'selected',
  'volunteer',
  'waitlisted',
]);
export const dutyHistoryStatus = pgEnum('duty_history_status', [
  'assigned',
  'cancelled',
  'completed',
  'incomplete',
  'reassigned',
]);
export const taskSource = pgEnum('task_source', ['generated', 'manual']);
export const taskStatus = pgEnum('task_status', ['completed', 'open']);

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

export const memberships = pgTable(
  'memberships',
  {
    id: uuid('id').defaultRandom().primaryKey(),
    jerseyNumber: integer('jersey_number'),
    participationType: participationType('participation_type').notNull(),
    playerId: uuid('player_id')
      .notNull()
      .references(() => players.id, { onDelete: 'cascade' }),
    relationship: membershipRelationship('relationship').notNull(),
    seasonId: uuid('season_id')
      .notNull()
      .references(() => seasons.id, { onDelete: 'cascade' }),
    status: membershipStatus('status').notNull(),
    teamId: uuid('team_id')
      .notNull()
      .references(() => teams.id, { onDelete: 'cascade' }),
    createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
  },
  (table) => [
    uniqueIndex('memberships_player_team_season_unique').on(
      table.playerId,
      table.teamId,
      table.seasonId,
    ),
  ],
);

export const locations = pgTable(
  'locations',
  {
    id: uuid('id').defaultRandom().primaryKey(),
    name: text('name').notNull(),
    travelMinutes: integer('travel_minutes').notNull(),
    createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
  },
  (table) => [uniqueIndex('locations_name_unique').on(table.name)],
);

export const trainingSeries = pgTable('training_series', {
  id: uuid('id').defaultRandom().primaryKey(),
  durationMinutes: integer('duration_minutes').notNull(),
  endDate: date('end_date', { mode: 'string' }).notNull(),
  locationId: uuid('location_id')
    .notNull()
    .references(() => locations.id),
  startDate: date('start_date', { mode: 'string' }).notNull(),
  startTime: text('start_time').notNull(),
  weekday: integer('weekday').notNull(),
  createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
});

export const trainingOccurrences = pgTable(
  'training_occurrences',
  {
    id: uuid('id').defaultRandom().primaryKey(),
    date: date('date', { mode: 'string' }).notNull(),
    durationMinutes: integer('duration_minutes').notNull(),
    locationId: uuid('location_id')
      .notNull()
      .references(() => locations.id),
    seriesId: uuid('series_id')
      .notNull()
      .references(() => trainingSeries.id, { onDelete: 'cascade' }),
    startTime: text('start_time').notNull(),
    createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
  },
  (table) => [
    uniqueIndex('training_occurrences_series_date_unique').on(table.seriesId, table.date),
  ],
);

export const gameFixtures = pgTable('game_fixtures', {
  id: uuid('id').defaultRandom().primaryKey(),
  awayTeamId: uuid('away_team_id')
    .notNull()
    .references(() => teams.id),
  homeTeamId: uuid('home_team_id')
    .notNull()
    .references(() => teams.id),
  createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
});

export const gameOccurrences = pgTable('game_occurrences', {
  id: uuid('id').defaultRandom().primaryKey(),
  arrivalBufferMinutes: integer('arrival_buffer_minutes').notNull(),
  date: date('date', { mode: 'string' }).notNull(),
  fixtureId: uuid('fixture_id')
    .notNull()
    .references(() => gameFixtures.id, { onDelete: 'cascade' }),
  locationId: uuid('location_id')
    .notNull()
    .references(() => locations.id),
  startTime: text('start_time').notNull(),
  status: gameOccurrenceStatus('status').notNull(),
  travelMinutes: integer('travel_minutes').notNull(),
  createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
});

export const dutyRequirements = pgTable(
  'duty_requirements',
  {
    id: uuid('id').defaultRandom().primaryKey(),
    drivingSlots: integer('driving_slots').notNull(),
    gameOccurrenceId: uuid('game_occurrence_id')
      .notNull()
      .references(() => gameOccurrences.id, { onDelete: 'cascade' }),
    jurySlots: integer('jury_slots').notNull(),
    refereeSlots: integer('referee_slots').notNull(),
    createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
  },
  (table) => [uniqueIndex('duty_requirements_occurrence_unique').on(table.gameOccurrenceId)],
);

export const dutySlots = pgTable(
  'duty_slots',
  {
    id: uuid('id').defaultRandom().primaryKey(),
    assignedPlayerId: uuid('assigned_player_id').references(() => players.id, {
      onDelete: 'set null',
    }),
    createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
    gameOccurrenceId: uuid('game_occurrence_id')
      .notNull()
      .references(() => gameOccurrences.id, { onDelete: 'cascade' }),
    slotNumber: integer('slot_number').notNull(),
    status: dutySlotStatus('status').notNull(),
    type: dutyType('type').notNull(),
  },
  (table) => [
    uniqueIndex('duty_slots_occurrence_type_number_unique').on(
      table.gameOccurrenceId,
      table.type,
      table.slotNumber,
    ),
  ],
);

export const dutySignups = pgTable(
  'duty_signups',
  {
    id: uuid('id').defaultRandom().primaryKey(),
    createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
    gameOccurrenceId: uuid('game_occurrence_id')
      .notNull()
      .references(() => gameOccurrences.id, { onDelete: 'cascade' }),
    playerId: uuid('player_id')
      .notNull()
      .references(() => players.id, { onDelete: 'cascade' }),
    status: dutySignupStatus('status').notNull(),
    type: dutyType('type').notNull(),
  },
  (table) => [
    uniqueIndex('duty_signups_occurrence_type_player_unique').on(
      table.gameOccurrenceId,
      table.type,
      table.playerId,
    ),
  ],
);

export const dutyAssignmentHistory = pgTable('duty_assignment_history', {
  id: uuid('id').defaultRandom().primaryKey(),
  createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
  playerId: uuid('player_id')
    .notNull()
    .references(() => players.id),
  slotId: uuid('slot_id')
    .notNull()
    .references(() => dutySlots.id, { onDelete: 'cascade' }),
  status: dutyHistoryStatus('status').notNull(),
});

export const tasks = pgTable('tasks', {
  completedAt: timestamp('completed_at', { mode: 'date', withTimezone: true }),
  createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
  description: text('description'),
  dueDate: date('due_date', { mode: 'string' }),
  id: uuid('id').defaultRandom().primaryKey(),
  occurrenceId: uuid('occurrence_id').references(() => gameOccurrences.id, {
    onDelete: 'set null',
  }),
  source: taskSource('source').notNull(),
  status: taskStatus('status').notNull(),
  title: text('title').notNull(),
});

export const participationRecords = pgTable(
  'participation_records',
  {
    id: uuid('id').defaultRandom().primaryKey(),
    absenceReason: absenceReason('absence_reason'),
    occurrenceId: uuid('occurrence_id').notNull(),
    occurrenceType: participationOccurrenceType('occurrence_type').notNull(),
    playerId: uuid('player_id')
      .notNull()
      .references(() => players.id, { onDelete: 'cascade' }),
    status: participationStatus('status').notNull(),
    recordedAt: timestamp('recorded_at', { mode: 'date', withTimezone: true })
      .defaultNow()
      .notNull(),
  },
  (table) => [
    uniqueIndex('participation_records_occurrence_player_unique').on(
      table.occurrenceId,
      table.occurrenceType,
      table.playerId,
    ),
  ],
);
