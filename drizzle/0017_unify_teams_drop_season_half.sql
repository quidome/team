ALTER TABLE "teams" ADD COLUMN "is_own_team" boolean NOT NULL DEFAULT false;
--> statement-breakpoint
UPDATE "teams" SET "is_own_team" = true;
--> statement-breakpoint
INSERT INTO "teams" ("name", "is_own_team")
  SELECT DISTINCT "name", false FROM "opponents"
  ON CONFLICT ("name") DO NOTHING;
--> statement-breakpoint
UPDATE "game_fixtures" gf SET
  "home_team_id" = CASE WHEN gf."is_home"
    THEN gf."our_team_id"
    ELSE (SELECT t."id" FROM "teams" t JOIN "opponents" o ON o."name" = t."name" WHERE o."id" = gf."opponent_id")
  END,
  "away_team_id" = CASE WHEN gf."is_home"
    THEN (SELECT t."id" FROM "teams" t JOIN "opponents" o ON o."name" = t."name" WHERE o."id" = gf."opponent_id")
    ELSE gf."our_team_id"
  END
WHERE gf."opponent_id" IS NOT NULL;
--> statement-breakpoint
ALTER TABLE "game_fixtures" ALTER COLUMN "home_team_id" SET NOT NULL;
--> statement-breakpoint
ALTER TABLE "game_fixtures" ALTER COLUMN "away_team_id" SET NOT NULL;
--> statement-breakpoint
ALTER TABLE "game_fixtures" DROP COLUMN "is_home";
--> statement-breakpoint
ALTER TABLE "game_fixtures" DROP COLUMN "opponent_id";
--> statement-breakpoint
ALTER TABLE "game_fixtures" DROP COLUMN "our_team_id";
--> statement-breakpoint
ALTER TABLE "game_fixtures" DROP COLUMN "season_half_id";
--> statement-breakpoint
DROP TABLE "opponent_season_halves";
--> statement-breakpoint
DROP TABLE "opponents";
--> statement-breakpoint
DROP TABLE "season_halves";
--> statement-breakpoint
DROP TYPE "public"."season_half_code";
