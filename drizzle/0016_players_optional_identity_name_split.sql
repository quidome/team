ALTER TABLE "players" ALTER COLUMN "association_id" DROP NOT NULL;
--> statement-breakpoint
ALTER TABLE "players" ALTER COLUMN "birth_date" DROP NOT NULL;
--> statement-breakpoint
ALTER TABLE "players" ADD COLUMN "first_name" text;
--> statement-breakpoint
UPDATE "players" SET "first_name" = "name";
--> statement-breakpoint
ALTER TABLE "players" ALTER COLUMN "first_name" SET NOT NULL;
--> statement-breakpoint
ALTER TABLE "players" ADD COLUMN "last_name" text;
--> statement-breakpoint
ALTER TABLE "players" DROP COLUMN "name";
