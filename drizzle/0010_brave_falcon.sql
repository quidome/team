CREATE TABLE "audit_entries" (
	"action" text NOT NULL,
	"entity_id" text NOT NULL,
	"entity_type" text NOT NULL,
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"metadata" jsonb NOT NULL,
	"occurred_at" timestamp with time zone DEFAULT now() NOT NULL
);
