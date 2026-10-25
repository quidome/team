-- A training is checked when the coordinator confirmed that its attendance is complete (story C of
-- start page and season). At the release every training before today (Dutch date) counts as
-- checked, so the start page does not show them all as open items.
ALTER TABLE trainings ADD COLUMN checked_at timestamptz;
UPDATE trainings SET checked_at = now()
WHERE date < (now() AT TIME ZONE 'Europe/Amsterdam')::date;
