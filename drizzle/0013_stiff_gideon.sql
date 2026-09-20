CREATE TYPE "public"."season_half_code" AS ENUM('H1', 'H2');--> statement-breakpoint
CREATE TABLE "opponent_season_halves" (
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"half_id" uuid NOT NULL,
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"opponent_id" uuid NOT NULL
);
--> statement-breakpoint
CREATE TABLE "opponents" (
	"address" text,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"name" text NOT NULL,
	"season_id" uuid NOT NULL,
	"travel_minutes" integer DEFAULT 0 NOT NULL
);
--> statement-breakpoint
CREATE TABLE "season_halves" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"half" "season_half_code" NOT NULL,
	"season_id" uuid NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "game_fixtures" ALTER COLUMN "away_team_id" DROP NOT NULL;--> statement-breakpoint
ALTER TABLE "game_fixtures" ALTER COLUMN "home_team_id" DROP NOT NULL;--> statement-breakpoint
ALTER TABLE "game_fixtures" ADD COLUMN "is_home" boolean DEFAULT true NOT NULL;--> statement-breakpoint
ALTER TABLE "game_fixtures" ADD COLUMN "opponent_id" uuid;--> statement-breakpoint
ALTER TABLE "game_fixtures" ADD COLUMN "our_team_id" uuid;--> statement-breakpoint
ALTER TABLE "game_fixtures" ADD COLUMN "season_half_id" uuid;--> statement-breakpoint
ALTER TABLE "opponent_season_halves" ADD CONSTRAINT "opponent_season_halves_half_id_season_halves_id_fk" FOREIGN KEY ("half_id") REFERENCES "public"."season_halves"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "opponent_season_halves" ADD CONSTRAINT "opponent_season_halves_opponent_id_opponents_id_fk" FOREIGN KEY ("opponent_id") REFERENCES "public"."opponents"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "opponents" ADD CONSTRAINT "opponents_season_id_seasons_id_fk" FOREIGN KEY ("season_id") REFERENCES "public"."seasons"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "season_halves" ADD CONSTRAINT "season_halves_season_id_seasons_id_fk" FOREIGN KEY ("season_id") REFERENCES "public"."seasons"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
CREATE UNIQUE INDEX "opponent_season_halves_unique" ON "opponent_season_halves" USING btree ("opponent_id","half_id");--> statement-breakpoint
CREATE UNIQUE INDEX "opponents_season_name_unique" ON "opponents" USING btree ("season_id","name");--> statement-breakpoint
CREATE UNIQUE INDEX "season_halves_season_half_unique" ON "season_halves" USING btree ("season_id","half");--> statement-breakpoint
ALTER TABLE "game_fixtures" ADD CONSTRAINT "game_fixtures_opponent_id_opponents_id_fk" FOREIGN KEY ("opponent_id") REFERENCES "public"."opponents"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "game_fixtures" ADD CONSTRAINT "game_fixtures_our_team_id_teams_id_fk" FOREIGN KEY ("our_team_id") REFERENCES "public"."teams"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "game_fixtures" ADD CONSTRAINT "game_fixtures_season_half_id_season_halves_id_fk" FOREIGN KEY ("season_half_id") REFERENCES "public"."season_halves"("id") ON DELETE cascade ON UPDATE no action;