CREATE TABLE "clubs" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"is_own_club" boolean DEFAULT false NOT NULL,
	"name" text NOT NULL,
	"source_club_id" integer,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "coordinator_teams" (
	"coordinator_id" uuid NOT NULL,
	"team_id" uuid NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "coordinators" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"default_team_id" uuid,
	"subject" text NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "teams" ADD COLUMN "club_id" uuid;--> statement-breakpoint
ALTER TABLE "coordinator_teams" ADD CONSTRAINT "coordinator_teams_coordinator_id_coordinators_id_fk" FOREIGN KEY ("coordinator_id") REFERENCES "public"."coordinators"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "coordinator_teams" ADD CONSTRAINT "coordinator_teams_team_id_teams_id_fk" FOREIGN KEY ("team_id") REFERENCES "public"."teams"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "coordinators" ADD CONSTRAINT "coordinators_default_team_id_teams_id_fk" FOREIGN KEY ("default_team_id") REFERENCES "public"."teams"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
CREATE UNIQUE INDEX "clubs_name_unique" ON "clubs" USING btree ("name");--> statement-breakpoint
CREATE UNIQUE INDEX "clubs_source_club_id_unique" ON "clubs" USING btree ("source_club_id");--> statement-breakpoint
CREATE UNIQUE INDEX "clubs_single_own_club" ON "clubs" USING btree ("is_own_club") WHERE "clubs"."is_own_club";--> statement-breakpoint
CREATE UNIQUE INDEX "coordinator_teams_coordinator_team_unique" ON "coordinator_teams" USING btree ("coordinator_id","team_id");--> statement-breakpoint
CREATE UNIQUE INDEX "coordinators_subject_unique" ON "coordinators" USING btree ("subject");--> statement-breakpoint
ALTER TABLE "teams" ADD CONSTRAINT "teams_club_id_clubs_id_fk" FOREIGN KEY ("club_id") REFERENCES "public"."clubs"("id") ON DELETE set null ON UPDATE no action;