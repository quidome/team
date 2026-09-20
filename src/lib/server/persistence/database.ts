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

export type Database = ReturnType<typeof createDatabase>;
export type DatabaseTransaction = Parameters<Parameters<Database['transaction']>[0]>[0];
export type DatabaseConnection = Database | DatabaseTransaction;
