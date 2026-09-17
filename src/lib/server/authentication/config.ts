type Environment = Readonly<Record<string, string | undefined>>;

export class ConfigurationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ConfigurationError';
  }
}

export interface AuthenticationConfiguration {
  applicationOrigin: string;
  clientId: string;
  clientSecret: string;
  issuerUrl: string;
  redirectUrl: string;
  sessionSecret: string;
  sessionTtlSeconds: number;
}

const readRequiredValue = (environment: Environment, name: string) => {
  const value = environment[name]?.trim();

  if (!value) {
    throw new ConfigurationError(`${name} must be set`);
  }

  return value;
};

const readUrl = (environment: Environment, name: string) => {
  const value = readRequiredValue(environment, name);

  try {
    return new URL(value);
  } catch {
    throw new ConfigurationError(`${name} must be a valid URL`);
  }
};

const readApplicationOrigin = (environment: Environment) => {
  const url = readUrl(environment, 'ORIGIN');

  if (url.origin !== url.toString().replace(/\/$/, '')) {
    throw new ConfigurationError('ORIGIN must not include a path, query, or fragment');
  }

  return url;
};

const readPositiveWholeNumber = (environment: Environment, name: string) => {
  const value = Number(readRequiredValue(environment, name));

  if (!Number.isSafeInteger(value) || value <= 0) {
    throw new ConfigurationError(`${name} must be a positive whole number`);
  }

  return value;
};

export const readAuthenticationConfiguration = (
  environment: Environment,
): AuthenticationConfiguration => {
  const applicationOrigin = readApplicationOrigin(environment);
  const issuerUrl = readUrl(environment, 'OIDC_ISSUER_URL');
  const sessionSecret = readRequiredValue(environment, 'SESSION_SECRET');

  if (sessionSecret.length < 32) {
    throw new ConfigurationError('SESSION_SECRET must be at least 32 characters');
  }

  return {
    applicationOrigin: applicationOrigin.origin,
    clientId: readRequiredValue(environment, 'OIDC_CLIENT_ID'),
    clientSecret: readRequiredValue(environment, 'OIDC_CLIENT_SECRET'),
    issuerUrl: issuerUrl.toString(),
    redirectUrl: new URL('/auth/callback', applicationOrigin).toString(),
    sessionSecret,
    sessionTtlSeconds: readPositiveWholeNumber(environment, 'SESSION_TTL_SECONDS'),
  };
};
