CREATE TYPE "public"."game_occurrence_status" AS ENUM('cancelled', 'scheduled');--> statement-breakpoint
CREATE TABLE "game_fixtures" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"away_team_id" uuid NOT NULL,
	"home_team_id" uuid NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "game_occurrences" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"arrival_buffer_minutes" integer NOT NULL,
	"date" date NOT NULL,
	"fixture_id" uuid NOT NULL,
	"location_id" uuid NOT NULL,
	"start_time" text NOT NULL,
	"status" "game_occurrence_status" NOT NULL,
	"travel_minutes" integer NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "game_fixtures" ADD CONSTRAINT "game_fixtures_away_team_id_teams_id_fk" FOREIGN KEY ("away_team_id") REFERENCES "public"."teams"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "game_fixtures" ADD CONSTRAINT "game_fixtures_home_team_id_teams_id_fk" FOREIGN KEY ("home_team_id") REFERENCES "public"."teams"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "game_occurrences" ADD CONSTRAINT "game_occurrences_fixture_id_game_fixtures_id_fk" FOREIGN KEY ("fixture_id") REFERENCES "public"."game_fixtures"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "game_occurrences" ADD CONSTRAINT "game_occurrences_location_id_locations_id_fk" FOREIGN KEY ("location_id") REFERENCES "public"."locations"("id") ON DELETE no action ON UPDATE no action;