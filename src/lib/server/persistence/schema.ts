import {
  boolean,
  date,
  integer,
  jsonb,
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
export const trainingOccurrenceStatus = pgEnum('training_occurrence_status', [
  'cancelled',
  'scheduled',
]);

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

export const seasonHalfCode = pgEnum('season_half_code', ['H1', 'H2']);

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

export const seasonHalves = pgTable(
  'season_halves',
  {
    id: uuid('id').defaultRandom().primaryKey(),
    half: seasonHalfCode('half').notNull(),
    seasonId: uuid('season_id')
      .notNull()
      .references(() => seasons.id, { onDelete: 'cascade' }),
    createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
  },
  (table) => [uniqueIndex('season_halves_season_half_unique').on(table.seasonId, table.half)],
);

export const opponents = pgTable(
  'opponents',
  {
    address: text('address'),
    createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
    id: uuid('id').defaultRandom().primaryKey(),
    name: text('name').notNull(),
    seasonId: uuid('season_id')
      .notNull()
      .references(() => seasons.id, { onDelete: 'cascade' }),
    travelMinutes: integer('travel_minutes').notNull().default(0),
  },
  (table) => [uniqueIndex('opponents_season_name_unique').on(table.seasonId, table.name)],
);

export const opponentSeasonHalves = pgTable(
  'opponent_season_halves',
  {
    createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
    halfId: uuid('half_id')
      .notNull()
      .references(() => seasonHalves.id, { onDelete: 'cascade' }),
    id: uuid('id').defaultRandom().primaryKey(),
    opponentId: uuid('opponent_id')
      .notNull()
      .references(() => opponents.id, { onDelete: 'cascade' }),
  },
  (table) => [uniqueIndex('opponent_season_halves_unique').on(table.opponentId, table.halfId)],
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
    .references(() => locations.id, { onDelete: 'cascade' }),
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
      .references(() => locations.id, { onDelete: 'cascade' }),
    seriesId: uuid('series_id').references(() => trainingSeries.id, { onDelete: 'cascade' }),
    startTime: text('start_time').notNull(),
    status: trainingOccurrenceStatus('status').notNull().default('scheduled'),
    createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
  },
  (table) => [
    uniqueIndex('training_occurrences_series_date_unique').on(table.seriesId, table.date),
  ],
);

export const gameFixtures = pgTable('game_fixtures', {
  awayTeamId: uuid('away_team_id').references(() => teams.id, { onDelete: 'cascade' }),
  createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
  homeTeamId: uuid('home_team_id').references(() => teams.id, { onDelete: 'cascade' }),
  id: uuid('id').defaultRandom().primaryKey(),
  isHome: boolean('is_home').notNull().default(true),
  opponentId: uuid('opponent_id').references(() => opponents.id, { onDelete: 'cascade' }),
  ourTeamId: uuid('our_team_id').references(() => teams.id, { onDelete: 'cascade' }),
  seasonHalfId: uuid('season_half_id').references(() => seasonHalves.id, { onDelete: 'cascade' }),
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
    .references(() => locations.id, { onDelete: 'cascade' }),
  startTime: text('start_time').notNull(),
  status: gameOccurrenceStatus('status').notNull(),
  travelMinutes: integer('travel_minutes').notNull(),
  createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
});

export const gameImportProvenance = pgTable(
  'game_import_provenance',
  {
    id: uuid('id').defaultRandom().primaryKey(),
    importedAt: timestamp('imported_at', { mode: 'date', withTimezone: true }).notNull(),
    occurrenceId: uuid('occurrence_id')
      .notNull()
      .references(() => gameOccurrences.id, { onDelete: 'cascade' }),
    sourceName: text('source_name').notNull(),
    sourceRow: integer('source_row').notNull(),
    createdAt: timestamp('created_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
  },
  (table) => [uniqueIndex('game_import_provenance_occurrence_unique').on(table.occurrenceId)],
);

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

export const auditEntries = pgTable('audit_entries', {
  action: text('action').notNull(),
  entityId: text('entity_id').notNull(),
  entityType: text('entity_type').notNull(),
  id: uuid('id').defaultRandom().primaryKey(),
  metadata: jsonb('metadata').$type<Record<string, boolean | number | string>>().notNull(),
  occurredAt: timestamp('occurred_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
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

export const coordinatorSettings = pgTable('coordinator_settings', {
  id: text('id').primaryKey().default('singleton'),
  primaryTeamId: uuid('primary_team_id')
    .notNull()
    .references(() => teams.id, { onDelete: 'restrict' }),
  seasonId: uuid('season_id')
    .notNull()
    .references(() => seasons.id, { onDelete: 'restrict' }),
  updatedAt: timestamp('updated_at', { mode: 'date', withTimezone: true }).defaultNow().notNull(),
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
