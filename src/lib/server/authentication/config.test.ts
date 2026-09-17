import { describe, expect, it } from 'vitest';

import { ConfigurationError, readAuthenticationConfiguration } from './config';

const validEnvironment = {
  ORIGIN: 'https://team.example.test',
  OIDC_CLIENT_ID: 'team-coordinator',
  OIDC_CLIENT_SECRET: 'client-secret',
  OIDC_ISSUER_URL: 'https://pocket-id.example.test',
  SESSION_SECRET: 'a-secure-session-secret-with-at-least-thirty-two-characters',
  SESSION_TTL_SECONDS: '28800',
};

describe('authentication configuration', () => {
  it('derives the registered callback URL from the configured application origin', () => {
    expect(readAuthenticationConfiguration(validEnvironment)).toMatchObject({
      clientId: 'team-coordinator',
      issuerUrl: 'https://pocket-id.example.test/',
      redirectUrl: 'https://team.example.test/auth/callback',
      sessionTtlSeconds: 28800,
    });
  });

  it('rejects a configuration without an OIDC client secret', () => {
    const environment = { ...validEnvironment, OIDC_CLIENT_SECRET: '' };

    expect(() => readAuthenticationConfiguration(environment)).toThrow(
      new ConfigurationError('OIDC_CLIENT_SECRET must be set'),
    );
  });

  it('rejects a short session secret', () => {
    const environment = { ...validEnvironment, SESSION_SECRET: 'too-short' };

    expect(() => readAuthenticationConfiguration(environment)).toThrow(
      new ConfigurationError('SESSION_SECRET must be at least 32 characters'),
    );
  });

  it('rejects a non-positive session lifetime', () => {
    const environment = { ...validEnvironment, SESSION_TTL_SECONDS: '0' };

    expect(() => readAuthenticationConfiguration(environment)).toThrow(
      new ConfigurationError('SESSION_TTL_SECONDS must be a positive whole number'),
    );
  });
});
