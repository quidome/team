import 'dotenv/config';

import { defineConfig } from 'drizzle-kit';

export default defineConfig({
  dbCredentials: {
    // Generation only reads the schema; migrations require a real DATABASE_URL.
    url:
      process.env.DATABASE_URL ?? 'postgresql://placeholder:placeholder@localhost:5432/placeholder',
  },
  dialect: 'postgresql',
  out: './drizzle',
  schema: './src/lib/server/persistence/schema.ts',
});
