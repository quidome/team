import * as oidcClient from 'openid-client';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { createOidcClient } from './oidc';

vi.mock('openid-client', () => ({
  authorizationCodeGrant: vi.fn(),
  buildAuthorizationUrl: vi.fn(),
  calculatePKCECodeChallenge: vi.fn(),
  discovery: vi.fn(),
  randomNonce: vi.fn(),
  randomPKCECodeVerifier: vi.fn(),
  randomState: vi.fn(),
}));

const configuration = {
  applicationOrigin: 'https://team.example.test',
  clientId: 'team-coordinator',
  clientSecret: 'client-secret',
  issuerUrl: 'https://pocket-id.example.test/',
  redirectUrl: 'https://team.example.test/auth/callback',
  sessionSecret: 'a-secure-session-secret-with-at-least-thirty-two-characters',
  sessionTtlSeconds: 28800,
};

const providerConfiguration = {} as oidcClient.Configuration;

describe('Pocket ID OIDC client', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    vi.mocked(oidcClient.discovery).mockResolvedValue(providerConfiguration);
    vi.mocked(oidcClient.randomPKCECodeVerifier).mockReturnValue('pkce-verifier');
    vi.mocked(oidcClient.randomNonce).mockReturnValue('oidc-nonce');
    vi.mocked(oidcClient.randomState).mockReturnValue('oidc-state');
    vi.mocked(oidcClient.calculatePKCECodeChallenge).mockResolvedValue('pkce-challenge');
    vi.mocked(oidcClient.buildAuthorizationUrl).mockReturnValue(
      new URL('https://pocket-id.example.test/authorize'),
    );
  });

  it('builds an authorization request with PKCE, state, and nonce', async () => {
    const client = createOidcClient(configuration);

    await expect(client.startAuthorization()).resolves.toEqual({
      transaction: {
        codeVerifier: 'pkce-verifier',
        nonce: 'oidc-nonce',
        state: 'oidc-state',
      },
      url: new URL('https://pocket-id.example.test/authorize'),
    });
    expect(oidcClient.buildAuthorizationUrl).toHaveBeenCalledWith(providerConfiguration, {
      code_challenge: 'pkce-challenge',
      code_challenge_method: 'S256',
      nonce: 'oidc-nonce',
      redirect_uri: 'https://team.example.test/auth/callback',
      scope: 'openid',
      state: 'oidc-state',
    });
  });

  it('returns the validated OpenID Connect subject after the callback', async () => {
    vi.mocked(oidcClient.authorizationCodeGrant).mockResolvedValue({
      claims: () => ({ sub: 'pocket-id-subject' }),
    } as never);
    const client = createOidcClient(configuration);

    await expect(
      client.completeAuthorization(new URL('https://team.example.test/auth/callback?code=code'), {
        codeVerifier: 'pkce-verifier',
        nonce: 'oidc-nonce',
        state: 'oidc-state',
      }),
    ).resolves.toEqual({ subject: 'pocket-id-subject' });
    expect(oidcClient.authorizationCodeGrant).toHaveBeenCalledWith(
      providerConfiguration,
      new URL('https://team.example.test/auth/callback?code=code'),
      {
        expectedNonce: 'oidc-nonce',
        expectedState: 'oidc-state',
        pkceCodeVerifier: 'pkce-verifier',
      },
    );
  });
});
