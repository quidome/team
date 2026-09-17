type Environment = Readonly<Record<string, string | undefined>>;

export class DatabaseConfigurationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'DatabaseConfigurationError';
  }
}

export const readDatabaseUrl = (environment: Environment): string => {
  const value = environment.DATABASE_URL?.trim();

  if (!value) {
    throw new DatabaseConfigurationError('DATABASE_URL must be set');
  }

  try {
    const url = new URL(value);

    if (url.protocol !== 'postgres:' && url.protocol !== 'postgresql:') {
      throw new DatabaseConfigurationError('DATABASE_URL must use the PostgreSQL protocol');
    }
  } catch (error) {
    if (error instanceof DatabaseConfigurationError) {
      throw error;
    }

    throw new DatabaseConfigurationError('DATABASE_URL must be a valid URL');
  }

  return value;
};
