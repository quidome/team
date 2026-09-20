CREATE TYPE "public"."task_source" AS ENUM('generated', 'manual');--> statement-breakpoint
CREATE TYPE "public"."task_status" AS ENUM('completed', 'open');--> statement-breakpoint
CREATE TABLE "tasks" (
	"completed_at" timestamp with time zone,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"description" text,
	"due_date" date,
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"occurrence_id" uuid,
	"source" "task_source" NOT NULL,
	"status" "task_status" NOT NULL,
	"title" text NOT NULL
);
--> statement-breakpoint
ALTER TABLE "tasks" ADD CONSTRAINT "tasks_occurrence_id_game_occurrences_id_fk" FOREIGN KEY ("occurrence_id") REFERENCES "public"."game_occurrences"("id") ON DELETE set null ON UPDATE no action;