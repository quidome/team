import { drizzle } from 'drizzle-orm/postgres-js';
import postgres from 'postgres';

import * as schema from './schema';

export const createDatabase = (databaseUrl: string) => {
  const client = postgres(databaseUrl);
  const database = drizzle({ client, schema });

  return Object.assign(database, {
    close: () => client.end({ timeout: 5 }),
  });
};
