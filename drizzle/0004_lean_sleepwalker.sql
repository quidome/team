CREATE TABLE "locations" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"name" text NOT NULL,
	"travel_minutes" integer NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "training_occurrences" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"date" date NOT NULL,
	"duration_minutes" integer NOT NULL,
	"location_id" uuid NOT NULL,
	"series_id" uuid NOT NULL,
	"start_time" text NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "training_series" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"duration_minutes" integer NOT NULL,
	"end_date" date NOT NULL,
	"location_id" uuid NOT NULL,
	"start_date" date NOT NULL,
	"start_time" text NOT NULL,
	"weekday" integer NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "training_occurrences" ADD CONSTRAINT "training_occurrences_location_id_locations_id_fk" FOREIGN KEY ("location_id") REFERENCES "public"."locations"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "training_occurrences" ADD CONSTRAINT "training_occurrences_series_id_training_series_id_fk" FOREIGN KEY ("series_id") REFERENCES "public"."training_series"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "training_series" ADD CONSTRAINT "training_series_location_id_locations_id_fk" FOREIGN KEY ("location_id") REFERENCES "public"."locations"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
CREATE UNIQUE INDEX "locations_name_unique" ON "locations" USING btree ("name");--> statement-breakpoint
CREATE UNIQUE INDEX "training_occurrences_series_date_unique" ON "training_occurrences" USING btree ("series_id","date");