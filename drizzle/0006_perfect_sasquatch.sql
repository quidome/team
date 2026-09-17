CREATE TYPE "public"."absence_reason" AS ENUM('illness', 'injury', 'other');--> statement-breakpoint
CREATE TYPE "public"."participation_occurrence_type" AS ENUM('game', 'training');--> statement-breakpoint
CREATE TYPE "public"."participation_status" AS ENUM('absent', 'present');--> statement-breakpoint
CREATE TABLE "participation_records" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"absence_reason" "absence_reason",
	"occurrence_id" uuid NOT NULL,
	"occurrence_type" "participation_occurrence_type" NOT NULL,
	"player_id" uuid NOT NULL,
	"status" "participation_status" NOT NULL,
	"recorded_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "participation_records" ADD CONSTRAINT "participation_records_player_id_players_id_fk" FOREIGN KEY ("player_id") REFERENCES "public"."players"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
CREATE UNIQUE INDEX "participation_records_occurrence_player_unique" ON "participation_records" USING btree ("occurrence_id","occurrence_type","player_id");