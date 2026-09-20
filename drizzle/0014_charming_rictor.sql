ALTER TABLE "game_fixtures" DROP CONSTRAINT "game_fixtures_away_team_id_teams_id_fk";
--> statement-breakpoint
ALTER TABLE "game_fixtures" DROP CONSTRAINT "game_fixtures_home_team_id_teams_id_fk";
--> statement-breakpoint
ALTER TABLE "game_fixtures" DROP CONSTRAINT "game_fixtures_our_team_id_teams_id_fk";
--> statement-breakpoint
ALTER TABLE "game_occurrences" DROP CONSTRAINT "game_occurrences_location_id_locations_id_fk";
--> statement-breakpoint
ALTER TABLE "training_occurrences" DROP CONSTRAINT "training_occurrences_location_id_locations_id_fk";
--> statement-breakpoint
ALTER TABLE "training_series" DROP CONSTRAINT "training_series_location_id_locations_id_fk";
--> statement-breakpoint
ALTER TABLE "game_fixtures" ADD CONSTRAINT "game_fixtures_away_team_id_teams_id_fk" FOREIGN KEY ("away_team_id") REFERENCES "public"."teams"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "game_fixtures" ADD CONSTRAINT "game_fixtures_home_team_id_teams_id_fk" FOREIGN KEY ("home_team_id") REFERENCES "public"."teams"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "game_fixtures" ADD CONSTRAINT "game_fixtures_our_team_id_teams_id_fk" FOREIGN KEY ("our_team_id") REFERENCES "public"."teams"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "game_occurrences" ADD CONSTRAINT "game_occurrences_location_id_locations_id_fk" FOREIGN KEY ("location_id") REFERENCES "public"."locations"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "training_occurrences" ADD CONSTRAINT "training_occurrences_location_id_locations_id_fk" FOREIGN KEY ("location_id") REFERENCES "public"."locations"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "training_series" ADD CONSTRAINT "training_series_location_id_locations_id_fk" FOREIGN KEY ("location_id") REFERENCES "public"."locations"("id") ON DELETE cascade ON UPDATE no action;