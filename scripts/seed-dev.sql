BEGIN;

INSERT INTO seasons (starting_year, ending_year)
VALUES (2026, 2027)
ON CONFLICT (starting_year) DO NOTHING;

INSERT INTO teams (is_own_team, name)
VALUES (true, 'U16-1')
ON CONFLICT (name) DO NOTHING;

INSERT INTO locations (name, travel_minutes)
VALUES ('Home court', 0), ('Away court', 25), ('North court', 35)
ON CONFLICT (name) DO NOTHING;

INSERT INTO players (association_id, birth_date, first_name)
VALUES
  ('dev-player-01', '2010-01-12', 'Dev Player 01'),
  ('dev-player-02', '2010-03-24', 'Dev Player 02'),
  ('dev-player-03', '2010-05-08', 'Dev Player 03'),
  ('dev-player-04', '2010-07-19', 'Dev Player 04'),
  ('dev-player-05', '2010-09-02', 'Dev Player 05'),
  ('dev-player-06', '2010-11-16', 'Dev Player 06'),
  ('dev-player-07', '2011-02-05', 'Dev Player 07'),
  ('dev-player-08', '2011-04-21', 'Dev Player 08'),
  ('dev-player-09', '2011-06-30', 'Dev Player 09'),
  ('dev-player-10', '2011-08-14', 'Dev Player 10')
ON CONFLICT (association_id) DO NOTHING;

INSERT INTO memberships (
  jersey_number,
  participation_type,
  player_id,
  relationship,
  season_id,
  status,
  team_id
)
SELECT
  row_number() OVER (ORDER BY players.association_id)::integer,
  'trains_and_plays',
  players.id,
  'primary',
  seasons.id,
  'active',
  teams.id
FROM players
CROSS JOIN seasons
CROSS JOIN teams
WHERE players.association_id LIKE 'dev-player-%'
  AND seasons.starting_year = 2026
  AND teams.name = 'U16-1'
ON CONFLICT (player_id, team_id, season_id) DO NOTHING;

-- Opponents are just teams with is_own_team = false; a fixture references them directly
-- by home/away team id, so there is no separate opponent or season-half table anymore.
INSERT INTO teams (is_own_team, name)
VALUES
  (false, 'Haarlem Ballers U15-4'),
  (false, 'Almere Tigers U15-2'),
  (false, 'Leiden Lions U15-1'),
  (false, 'Rotterdam Rockets U15-3')
ON CONFLICT (name) DO NOTHING;

-- game_fixtures has no unique constraint on (home_team_id, away_team_id) — the app creates
-- a fresh fixture per imported game row — so idempotency here is checked via the occurrence's
-- date instead, one fixture+occurrence pair per game.
DO $$
DECLARE
  u16_team_id uuid;
  opponent_team_id uuid;
  location_id_value uuid;
  fixture_id_value uuid;
BEGIN
  SELECT id INTO u16_team_id FROM teams WHERE name = 'U16-1';
  SELECT id INTO location_id_value FROM locations WHERE name = 'Home court';

  -- Haarlem returns in both halves, both times at home.
  SELECT id INTO opponent_team_id FROM teams WHERE name = 'Haarlem Ballers U15-4';

  IF NOT EXISTS (
    SELECT 1 FROM game_occurrences go
    JOIN game_fixtures gf ON gf.id = go.fixture_id
    WHERE gf.home_team_id = u16_team_id AND gf.away_team_id = opponent_team_id
      AND go.date = '2026-08-15'
  ) THEN
    INSERT INTO game_fixtures (away_team_id, home_team_id)
    VALUES (opponent_team_id, u16_team_id)
    RETURNING id INTO fixture_id_value;
    INSERT INTO game_occurrences (arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes)
    VALUES (30, '2026-08-15', fixture_id_value, location_id_value, '14:30', 'scheduled', 0);
  END IF;

  IF NOT EXISTS (
    SELECT 1 FROM game_occurrences go
    JOIN game_fixtures gf ON gf.id = go.fixture_id
    WHERE gf.home_team_id = u16_team_id AND gf.away_team_id = opponent_team_id
      AND go.date = '2027-01-09'
  ) THEN
    INSERT INTO game_fixtures (away_team_id, home_team_id)
    VALUES (opponent_team_id, u16_team_id)
    RETURNING id INTO fixture_id_value;
    INSERT INTO game_occurrences (arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes)
    VALUES (30, '2027-01-09', fixture_id_value, location_id_value, '14:00', 'scheduled', 0);
  END IF;

  -- Almere returns in both halves, both times away.
  SELECT id INTO opponent_team_id FROM teams WHERE name = 'Almere Tigers U15-2';
  SELECT id INTO location_id_value FROM locations WHERE name = 'Away court';

  IF NOT EXISTS (
    SELECT 1 FROM game_occurrences go
    JOIN game_fixtures gf ON gf.id = go.fixture_id
    WHERE gf.away_team_id = u16_team_id AND gf.home_team_id = opponent_team_id
      AND go.date = '2026-08-22'
  ) THEN
    INSERT INTO game_fixtures (away_team_id, home_team_id)
    VALUES (u16_team_id, opponent_team_id)
    RETURNING id INTO fixture_id_value;
    INSERT INTO game_occurrences (arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes)
    VALUES (30, '2026-08-22', fixture_id_value, location_id_value, '15:00', 'scheduled', 25);
  END IF;

  IF NOT EXISTS (
    SELECT 1 FROM game_occurrences go
    JOIN game_fixtures gf ON gf.id = go.fixture_id
    WHERE gf.away_team_id = u16_team_id AND gf.home_team_id = opponent_team_id
      AND go.date = '2027-01-16'
  ) THEN
    INSERT INTO game_fixtures (away_team_id, home_team_id)
    VALUES (u16_team_id, opponent_team_id)
    RETURNING id INTO fixture_id_value;
    INSERT INTO game_occurrences (arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes)
    VALUES (30, '2027-01-16', fixture_id_value, location_id_value, '16:00', 'scheduled', 25);
  END IF;

  -- Leiden, two home games in different halves at a different court.
  SELECT id INTO opponent_team_id FROM teams WHERE name = 'Leiden Lions U15-1';
  SELECT id INTO location_id_value FROM locations WHERE name = 'North court';

  IF NOT EXISTS (
    SELECT 1 FROM game_occurrences go
    JOIN game_fixtures gf ON gf.id = go.fixture_id
    WHERE gf.home_team_id = u16_team_id AND gf.away_team_id = opponent_team_id
      AND go.date = '2026-09-05'
  ) THEN
    INSERT INTO game_fixtures (away_team_id, home_team_id)
    VALUES (opponent_team_id, u16_team_id)
    RETURNING id INTO fixture_id_value;
    INSERT INTO game_occurrences (arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes)
    VALUES (30, '2026-09-05', fixture_id_value, location_id_value, '11:00', 'scheduled', 35);
  END IF;

  IF NOT EXISTS (
    SELECT 1 FROM game_occurrences go
    JOIN game_fixtures gf ON gf.id = go.fixture_id
    WHERE gf.home_team_id = u16_team_id AND gf.away_team_id = opponent_team_id
      AND go.date = '2027-01-30'
  ) THEN
    INSERT INTO game_fixtures (away_team_id, home_team_id)
    VALUES (opponent_team_id, u16_team_id)
    RETURNING id INTO fixture_id_value;
    INSERT INTO game_occurrences (arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes)
    VALUES (30, '2027-01-30', fixture_id_value, location_id_value, '11:30', 'scheduled', 35);
  END IF;

  -- Rotterdam, a single H2 home game back at the home court.
  SELECT id INTO opponent_team_id FROM teams WHERE name = 'Rotterdam Rockets U15-3';
  SELECT id INTO location_id_value FROM locations WHERE name = 'Home court';

  IF NOT EXISTS (
    SELECT 1 FROM game_occurrences go
    JOIN game_fixtures gf ON gf.id = go.fixture_id
    WHERE gf.home_team_id = u16_team_id AND gf.away_team_id = opponent_team_id
      AND go.date = '2027-02-13'
  ) THEN
    INSERT INTO game_fixtures (away_team_id, home_team_id)
    VALUES (opponent_team_id, u16_team_id)
    RETURNING id INTO fixture_id_value;
    INSERT INTO game_occurrences (arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes)
    VALUES (30, '2027-02-13', fixture_id_value, location_id_value, '15:00', 'scheduled', 0);
  END IF;
END $$;

COMMIT;
