CREATE TABLE "game_import_provenance" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"imported_at" timestamp with time zone NOT NULL,
	"occurrence_id" uuid NOT NULL,
	"source_name" text NOT NULL,
	"source_row" integer NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "game_import_provenance" ADD CONSTRAINT "game_import_provenance_occurrence_id_game_occurrences_id_fk" FOREIGN KEY ("occurrence_id") REFERENCES "public"."game_occurrences"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
CREATE UNIQUE INDEX "game_import_provenance_occurrence_unique" ON "game_import_provenance" USING btree ("occurrence_id");