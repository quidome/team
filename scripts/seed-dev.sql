BEGIN;

INSERT INTO seasons (starting_year, ending_year)
VALUES (2026, 2027)
ON CONFLICT (starting_year) DO NOTHING;

INSERT INTO teams (name)
VALUES ('U16-1'), ('U18-1'), ('U18-2'), ('U20-1')
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

DO $$
DECLARE
  u16_team_id uuid;
  opponent_team_id uuid;
  home_location_id uuid;
  away_location_id uuid;
  north_location_id uuid;
  fixture_id_value uuid;
BEGIN
  SELECT id INTO u16_team_id FROM teams WHERE name = 'U16-1';
  SELECT id INTO home_location_id FROM locations WHERE name = 'Home court';
  SELECT id INTO away_location_id FROM locations WHERE name = 'Away court';
  SELECT id INTO north_location_id FROM locations WHERE name = 'North court';

  SELECT id INTO opponent_team_id FROM teams WHERE name = 'U18-1';
  SELECT gf.id INTO fixture_id_value
  FROM game_fixtures gf
  WHERE gf.home_team_id = u16_team_id AND gf.away_team_id = opponent_team_id
  LIMIT 1;
  IF fixture_id_value IS NULL THEN
    INSERT INTO game_fixtures (home_team_id, away_team_id)
    VALUES (u16_team_id, opponent_team_id)
    RETURNING id INTO fixture_id_value;
  END IF;
  IF NOT EXISTS (
    SELECT 1 FROM game_occurrences go
    WHERE go.fixture_id = fixture_id_value AND go.date = '2026-08-15'
  ) THEN
    INSERT INTO game_occurrences (
      arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes
    ) VALUES (30, '2026-08-15', fixture_id_value, home_location_id, '14:30', 'scheduled', 0);
  END IF;

  SELECT id INTO opponent_team_id FROM teams WHERE name = 'U18-2';
  fixture_id_value := NULL;
  SELECT gf.id INTO fixture_id_value
  FROM game_fixtures gf
  WHERE gf.home_team_id = opponent_team_id AND gf.away_team_id = u16_team_id
  LIMIT 1;
  IF fixture_id_value IS NULL THEN
    INSERT INTO game_fixtures (home_team_id, away_team_id)
    VALUES (opponent_team_id, u16_team_id)
    RETURNING id INTO fixture_id_value;
  END IF;
  IF NOT EXISTS (
    SELECT 1 FROM game_occurrences go
    WHERE go.fixture_id = fixture_id_value AND go.date = '2026-08-22'
  ) THEN
    INSERT INTO game_occurrences (
      arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes
    ) VALUES (30, '2026-08-22', fixture_id_value, away_location_id, '15:00', 'scheduled', 25);
  END IF;

  SELECT id INTO opponent_team_id FROM teams WHERE name = 'U20-1';
  fixture_id_value := NULL;
  SELECT gf.id INTO fixture_id_value
  FROM game_fixtures gf
  WHERE gf.home_team_id = u16_team_id AND gf.away_team_id = opponent_team_id
  LIMIT 1;
  IF fixture_id_value IS NULL THEN
    INSERT INTO game_fixtures (home_team_id, away_team_id)
    VALUES (u16_team_id, opponent_team_id)
    RETURNING id INTO fixture_id_value;
  END IF;
  IF NOT EXISTS (
    SELECT 1 FROM game_occurrences go
    WHERE go.fixture_id = fixture_id_value AND go.date = '2026-09-05'
  ) THEN
    INSERT INTO game_occurrences (
      arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes
    ) VALUES (30, '2026-09-05', fixture_id_value, north_location_id, '11:00', 'scheduled', 35);
  END IF;

  SELECT id INTO opponent_team_id FROM teams WHERE name = 'U18-1';
  fixture_id_value := NULL;
  SELECT gf.id INTO fixture_id_value
  FROM game_fixtures gf
  WHERE gf.home_team_id = u16_team_id AND gf.away_team_id = opponent_team_id
  LIMIT 1;
  IF fixture_id_value IS NULL THEN
    INSERT INTO game_fixtures (home_team_id, away_team_id)
    VALUES (u16_team_id, opponent_team_id)
    RETURNING id INTO fixture_id_value;
  END IF;
  IF NOT EXISTS (
    SELECT 1 FROM game_occurrences go
    WHERE go.fixture_id = fixture_id_value AND go.date = '2027-01-09'
  ) THEN
    INSERT INTO game_occurrences (
      arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes
    ) VALUES (30, '2027-01-09', fixture_id_value, home_location_id, '14:00', 'scheduled', 0);
  END IF;

  SELECT id INTO opponent_team_id FROM teams WHERE name = 'U18-2';
  fixture_id_value := NULL;
  SELECT gf.id INTO fixture_id_value
  FROM game_fixtures gf
  WHERE gf.home_team_id = opponent_team_id AND gf.away_team_id = u16_team_id
  LIMIT 1;
  IF fixture_id_value IS NULL THEN
    INSERT INTO game_fixtures (home_team_id, away_team_id)
    VALUES (opponent_team_id, u16_team_id)
    RETURNING id INTO fixture_id_value;
  END IF;
  IF NOT EXISTS (
    SELECT 1 FROM game_occurrences go
    WHERE go.fixture_id = fixture_id_value AND go.date = '2027-01-16'
  ) THEN
    INSERT INTO game_occurrences (
      arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes
    ) VALUES (30, '2027-01-16', fixture_id_value, away_location_id, '16:00', 'scheduled', 25);
  END IF;

  SELECT id INTO opponent_team_id FROM teams WHERE name = 'U20-1';
  fixture_id_value := NULL;
  SELECT gf.id INTO fixture_id_value
  FROM game_fixtures gf
  WHERE gf.home_team_id = u16_team_id AND gf.away_team_id = opponent_team_id
  LIMIT 1;
  IF fixture_id_value IS NULL THEN
    INSERT INTO game_fixtures (home_team_id, away_team_id)
    VALUES (u16_team_id, opponent_team_id)
    RETURNING id INTO fixture_id_value;
  END IF;
  IF NOT EXISTS (
    SELECT 1 FROM game_occurrences go
    WHERE go.fixture_id = fixture_id_value AND go.date = '2027-01-30'
  ) THEN
    INSERT INTO game_occurrences (
      arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes
    ) VALUES (30, '2027-01-30', fixture_id_value, north_location_id, '11:30', 'scheduled', 35);
  END IF;

  SELECT id INTO opponent_team_id FROM teams WHERE name = 'U18-2';
  fixture_id_value := NULL;
  SELECT gf.id INTO fixture_id_value
  FROM game_fixtures gf
  WHERE gf.home_team_id = u16_team_id AND gf.away_team_id = opponent_team_id
  LIMIT 1;
  IF fixture_id_value IS NULL THEN
    INSERT INTO game_fixtures (home_team_id, away_team_id)
    VALUES (u16_team_id, opponent_team_id)
    RETURNING id INTO fixture_id_value;
  END IF;
  IF NOT EXISTS (
    SELECT 1 FROM game_occurrences go
    WHERE go.fixture_id = fixture_id_value AND go.date = '2027-02-13'
  ) THEN
    INSERT INTO game_occurrences (
      arrival_buffer_minutes, date, fixture_id, location_id, start_time, status, travel_minutes
    ) VALUES (30, '2027-02-13', fixture_id_value, home_location_id, '15:00', 'scheduled', 0);
  END IF;
END $$;

COMMIT;
