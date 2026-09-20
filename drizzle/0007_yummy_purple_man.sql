CREATE TYPE "public"."duty_history_status" AS ENUM('assigned', 'cancelled', 'completed', 'incomplete', 'reassigned');--> statement-breakpoint
CREATE TYPE "public"."duty_signup_status" AS ENUM('selected', 'volunteer', 'waitlisted');--> statement-breakpoint
CREATE TYPE "public"."duty_slot_status" AS ENUM('assigned', 'cancelled', 'completed', 'incomplete', 'open');--> statement-breakpoint
CREATE TYPE "public"."duty_type" AS ENUM('driving', 'jury', 'referee');--> statement-breakpoint
CREATE TABLE "duty_assignment_history" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"player_id" uuid NOT NULL,
	"slot_id" uuid NOT NULL,
	"status" "duty_history_status" NOT NULL
);
--> statement-breakpoint
CREATE TABLE "duty_requirements" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"driving_slots" integer NOT NULL,
	"game_occurrence_id" uuid NOT NULL,
	"jury_slots" integer NOT NULL,
	"referee_slots" integer NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "duty_signups" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"game_occurrence_id" uuid NOT NULL,
	"player_id" uuid NOT NULL,
	"status" "duty_signup_status" NOT NULL,
	"type" "duty_type" NOT NULL
);
--> statement-breakpoint
CREATE TABLE "duty_slots" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"assigned_player_id" uuid,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"game_occurrence_id" uuid NOT NULL,
	"slot_number" integer NOT NULL,
	"status" "duty_slot_status" NOT NULL,
	"type" "duty_type" NOT NULL
);
--> statement-breakpoint
ALTER TABLE "duty_assignment_history" ADD CONSTRAINT "duty_assignment_history_player_id_players_id_fk" FOREIGN KEY ("player_id") REFERENCES "public"."players"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "duty_assignment_history" ADD CONSTRAINT "duty_assignment_history_slot_id_duty_slots_id_fk" FOREIGN KEY ("slot_id") REFERENCES "public"."duty_slots"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "duty_requirements" ADD CONSTRAINT "duty_requirements_game_occurrence_id_game_occurrences_id_fk" FOREIGN KEY ("game_occurrence_id") REFERENCES "public"."game_occurrences"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "duty_signups" ADD CONSTRAINT "duty_signups_game_occurrence_id_game_occurrences_id_fk" FOREIGN KEY ("game_occurrence_id") REFERENCES "public"."game_occurrences"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "duty_signups" ADD CONSTRAINT "duty_signups_player_id_players_id_fk" FOREIGN KEY ("player_id") REFERENCES "public"."players"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "duty_slots" ADD CONSTRAINT "duty_slots_assigned_player_id_players_id_fk" FOREIGN KEY ("assigned_player_id") REFERENCES "public"."players"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "duty_slots" ADD CONSTRAINT "duty_slots_game_occurrence_id_game_occurrences_id_fk" FOREIGN KEY ("game_occurrence_id") REFERENCES "public"."game_occurrences"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
CREATE UNIQUE INDEX "duty_requirements_occurrence_unique" ON "duty_requirements" USING btree ("game_occurrence_id");--> statement-breakpoint
CREATE UNIQUE INDEX "duty_signups_occurrence_type_player_unique" ON "duty_signups" USING btree ("game_occurrence_id","type","player_id");--> statement-breakpoint
CREATE UNIQUE INDEX "duty_slots_occurrence_type_number_unique" ON "duty_slots" USING btree ("game_occurrence_id","type","slot_number");