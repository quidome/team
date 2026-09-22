CREATE TABLE "coordinator_settings" (
	"id" text PRIMARY KEY DEFAULT 'singleton' NOT NULL,
	"primary_team_id" uuid NOT NULL,
	"season_id" uuid NOT NULL,
	"updated_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "coordinator_settings" ADD CONSTRAINT "coordinator_settings_primary_team_id_teams_id_fk" FOREIGN KEY ("primary_team_id") REFERENCES "public"."teams"("id") ON DELETE restrict ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "coordinator_settings" ADD CONSTRAINT "coordinator_settings_season_id_seasons_id_fk" FOREIGN KEY ("season_id") REFERENCES "public"."seasons"("id") ON DELETE restrict ON UPDATE no action;