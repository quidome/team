BEGIN;

INSERT INTO seasons (starting_year, ending_year)
VALUES (2026, 2027)
ON CONFLICT (starting_year) DO NOTHING;

INSERT INTO teams (name)
VALUES ('U16-1')
ON CONFLICT (name) DO NOTHING;

INSERT INTO locations (name, travel_minutes)
VALUES ('Home court', 0), ('Away court', 25), ('North court', 35)
ON CONFLICT (name) DO NOTHING;

INSERT INTO players (association_id, birth_date, name)
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

INSERT INTO season_halves (half, season_id)
SELECT half, seasons.id
FROM seasons
CROSS JOIN (VALUES ('H1'::season_half_code), ('H2'::season_half_code)) AS halves(half)
WHERE seasons.starting_year = 2026
ON CONFLICT (season_id, half) DO NOTHING;

DO $$
DECLARE
  season_id_value uuid;
  u16_team_id uuid;
  h1_id uuid;
  h2_id uuid;
  opponent_id_value uuid;
  fixture_id_value uuid;
  location_id_value uuid;
BEGIN
  SELECT id INTO season_id_value FROM seasons WHERE starting_year = 2026;
  SELECT id INTO u16_team_id FROM teams WHERE name = 'U16-1';
  SELECT id INTO h1_id FROM season_halves WHERE season_id = season_id_value AND half = 'H1';
  SELECT id INTO h2_id FROM season_halves WHERE season_id = season_id_value AND half = 'H2';

  -- Haarlem returns in both halves. It is one opponent with two half memberships.
  INSERT INTO opponents (address, name, season_id, travel_minutes)
  VALUES ('Haarlem sports park', 'Haarlem Ballers U15-4', season_id_value, 25)
  ON CONFLICT (season_id, name) DO UPDATE
    SET address = EXCLUDED.address, travel_minutes = EXCLUDED.travel_minutes
  RETURNING id INTO opponent_id_value;
  INSERT INTO opponent_season_halves (half_id, opponent_id)
  VALUES (h1_id, opponent_id_value), (h2_id, opponent_id_value)
  ON CONFLICT (opponent_id, half_id) DO NOTHING;

  -- H1 home game.
  SELECT id INTO location_id_value FROM locations WHERE name = 'Home court';
  SELECT gf.id INTO fixture_id_value
  FROM game_fixtures gf
  WHERE gf.our_team_id = u16_team_id
    AND gf.opponent_id = opponent_id_value
    AND gf.season_half_id = h1_id
    AND gf.is_home = true
  LIMIT 1;
  IF fixture_id_value IS NULL THEN
    INSERT INTO game_fixtures (home_team_id, is_home, opponent_id, our_team_id, season_half_id)
    VALUES (u16_team_id, true, opponent_id_value, u16_team_id, h1_id)
    RETURNING id INTO fixture_id_value;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM game_occurrences WHERE fixture_id = fixture_id_value AND date = '2026-08-15') THEN
    INSERT INTO game_occurrences (arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes)
    VALUES (30, '2026-08-15', fixture_id_value, location_id_value, '14:30', 'scheduled', 0);
  END IF;

  -- H2 home game against the same opponent.
  fixture_id_value := NULL;
  SELECT gf.id INTO fixture_id_value
  FROM game_fixtures gf
  WHERE gf.our_team_id = u16_team_id
    AND gf.opponent_id = opponent_id_value
    AND gf.season_half_id = h2_id
    AND gf.is_home = true
  LIMIT 1;
  IF fixture_id_value IS NULL THEN
    INSERT INTO game_fixtures (home_team_id, is_home, opponent_id, our_team_id, season_half_id)
    VALUES (u16_team_id, true, opponent_id_value, u16_team_id, h2_id)
    RETURNING id INTO fixture_id_value;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM game_occurrences WHERE fixture_id = fixture_id_value AND date = '2027-01-09') THEN
    INSERT INTO game_occurrences (arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes)
    VALUES (30, '2027-01-09', fixture_id_value, location_id_value, '14:00', 'scheduled', 0);
  END IF;

  -- A second opponent returns in both halves, this time for away games.
  INSERT INTO opponents (address, name, season_id, travel_minutes)
  VALUES ('Almere sports centre', 'Almere Tigers U15-2', season_id_value, 25)
  ON CONFLICT (season_id, name) DO UPDATE
    SET address = EXCLUDED.address, travel_minutes = EXCLUDED.travel_minutes
  RETURNING id INTO opponent_id_value;
  INSERT INTO opponent_season_halves (half_id, opponent_id)
  VALUES (h1_id, opponent_id_value), (h2_id, opponent_id_value)
  ON CONFLICT (opponent_id, half_id) DO NOTHING;

  SELECT id INTO location_id_value FROM locations WHERE name = 'Away court';
  SELECT gf.id INTO fixture_id_value
  FROM game_fixtures gf
  WHERE gf.our_team_id = u16_team_id
    AND gf.opponent_id = opponent_id_value
    AND gf.season_half_id = h1_id
    AND gf.is_home = false
  LIMIT 1;
  IF fixture_id_value IS NULL THEN
    INSERT INTO game_fixtures (away_team_id, is_home, opponent_id, our_team_id, season_half_id)
    VALUES (u16_team_id, false, opponent_id_value, u16_team_id, h1_id)
    RETURNING id INTO fixture_id_value;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM game_occurrences WHERE fixture_id = fixture_id_value AND date = '2026-08-22') THEN
    INSERT INTO game_occurrences (arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes)
    VALUES (30, '2026-08-22', fixture_id_value, location_id_value, '15:00', 'scheduled', 25);
  END IF;

  fixture_id_value := NULL;
  SELECT gf.id INTO fixture_id_value
  FROM game_fixtures gf
  WHERE gf.our_team_id = u16_team_id
    AND gf.opponent_id = opponent_id_value
    AND gf.season_half_id = h2_id
    AND gf.is_home = false
  LIMIT 1;
  IF fixture_id_value IS NULL THEN
    INSERT INTO game_fixtures (away_team_id, is_home, opponent_id, our_team_id, season_half_id)
    VALUES (u16_team_id, false, opponent_id_value, u16_team_id, h2_id)
    RETURNING id INTO fixture_id_value;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM game_occurrences WHERE fixture_id = fixture_id_value AND date = '2027-01-16') THEN
    INSERT INTO game_occurrences (arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes)
    VALUES (30, '2027-01-16', fixture_id_value, location_id_value, '16:00', 'scheduled', 25);
  END IF;

  -- Additional H1 and H2 opponents.
  INSERT INTO opponents (address, name, season_id, travel_minutes)
  VALUES ('Leiden north sports hall', 'Leiden Lions U15-1', season_id_value, 35)
  ON CONFLICT (season_id, name) DO UPDATE
    SET address = EXCLUDED.address, travel_minutes = EXCLUDED.travel_minutes
  RETURNING id INTO opponent_id_value;
  INSERT INTO opponent_season_halves (half_id, opponent_id)
  VALUES (h1_id, opponent_id_value), (h2_id, opponent_id_value)
  ON CONFLICT (opponent_id, half_id) DO NOTHING;

  SELECT id INTO location_id_value FROM locations WHERE name = 'North court';
  fixture_id_value := NULL;
  INSERT INTO game_fixtures (home_team_id, is_home, opponent_id, our_team_id, season_half_id)
  SELECT u16_team_id, true, opponent_id_value, u16_team_id, h1_id
  WHERE NOT EXISTS (
    SELECT 1 FROM game_fixtures
    WHERE our_team_id = u16_team_id AND opponent_id = opponent_id_value AND season_half_id = h1_id
  )
  RETURNING id INTO fixture_id_value;
  IF fixture_id_value IS NULL THEN
    SELECT id INTO fixture_id_value
    FROM game_fixtures
    WHERE our_team_id = u16_team_id AND opponent_id = opponent_id_value AND season_half_id = h1_id
    LIMIT 1;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM game_occurrences WHERE fixture_id = fixture_id_value AND date = '2026-09-05') THEN
    INSERT INTO game_occurrences (arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes)
    VALUES (30, '2026-09-05', fixture_id_value, location_id_value, '11:00', 'scheduled', 35);
  END IF;

  fixture_id_value := NULL;
  INSERT INTO game_fixtures (home_team_id, is_home, opponent_id, our_team_id, season_half_id)
  SELECT u16_team_id, true, opponent_id_value, u16_team_id, h2_id
  WHERE NOT EXISTS (
    SELECT 1 FROM game_fixtures
    WHERE our_team_id = u16_team_id AND opponent_id = opponent_id_value AND season_half_id = h2_id
  )
  RETURNING id INTO fixture_id_value;
  IF fixture_id_value IS NULL THEN
    SELECT id INTO fixture_id_value
    FROM game_fixtures
    WHERE our_team_id = u16_team_id AND opponent_id = opponent_id_value AND season_half_id = h2_id
    LIMIT 1;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM game_occurrences WHERE fixture_id = fixture_id_value AND date = '2027-01-30') THEN
    INSERT INTO game_occurrences (arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes)
    VALUES (30, '2027-01-30', fixture_id_value, location_id_value, '11:30', 'scheduled', 35);
  END IF;

  INSERT INTO opponents (address, name, season_id, travel_minutes)
  VALUES ('Rotterdam west sports hall', 'Rotterdam Rockets U15-3', season_id_value, 25)
  ON CONFLICT (season_id, name) DO UPDATE
    SET address = EXCLUDED.address, travel_minutes = EXCLUDED.travel_minutes
  RETURNING id INTO opponent_id_value;
  INSERT INTO opponent_season_halves (half_id, opponent_id)
  VALUES (h2_id, opponent_id_value)
  ON CONFLICT (opponent_id, half_id) DO NOTHING;

  SELECT id INTO location_id_value FROM locations WHERE name = 'Home court';
  fixture_id_value := NULL;
  INSERT INTO game_fixtures (home_team_id, is_home, opponent_id, our_team_id, season_half_id)
  SELECT u16_team_id, true, opponent_id_value, u16_team_id, h2_id
  WHERE NOT EXISTS (
    SELECT 1 FROM game_fixtures
    WHERE our_team_id = u16_team_id AND opponent_id = opponent_id_value AND season_half_id = h2_id
  )
  RETURNING id INTO fixture_id_value;
  IF fixture_id_value IS NULL THEN
    SELECT id INTO fixture_id_value
    FROM game_fixtures
    WHERE our_team_id = u16_team_id AND opponent_id = opponent_id_value AND season_half_id = h2_id
    LIMIT 1;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM game_occurrences WHERE fixture_id = fixture_id_value AND date = '2027-02-13') THEN
    INSERT INTO game_occurrences (arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes)
    VALUES (30, '2027-02-13', fixture_id_value, location_id_value, '15:00', 'scheduled', 0);
  END IF;
END $$;

COMMIT;
