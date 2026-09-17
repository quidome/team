import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  authorizationTransactionCookieName,
  readAuthorizationTransaction,
} from '$lib/server/authentication/transaction';

const mocks = vi.hoisted(() => ({
  startAuthorization: vi.fn(),
}));

vi.mock('$env/dynamic/private', () => ({
  env: {
    ORIGIN: 'https://team.example.test',
    OIDC_CLIENT_ID: 'team-coordinator',
    OIDC_CLIENT_SECRET: 'client-secret',
    OIDC_ISSUER_URL: 'https://pocket-id.example.test',
    SESSION_SECRET: 'a-secure-session-secret-with-at-least-thirty-two-characters',
    SESSION_TTL_SECONDS: '28800',
  },
}));

vi.mock('$lib/server/authentication/oidc', () => ({
  createOidcClient: () => ({
    startAuthorization: mocks.startAuthorization,
  }),
}));

import { GET } from '../../../routes/auth/login/+server';

describe('GET /auth/login', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.startAuthorization.mockResolvedValue({
      transaction: {
        codeVerifier: 'pkce-verifier',
        nonce: 'oidc-nonce',
        state: 'oidc-state',
      },
      url: new URL('https://pocket-id.example.test/authorize'),
    });
  });

  it('stores a short-lived HTTP-only authorization transaction before redirecting', async () => {
    const cookies = { set: vi.fn() };

    await expect(GET({ cookies } as never)).rejects.toMatchObject({
      location: 'https://pocket-id.example.test/authorize',
      status: 303,
    });
    expect(mocks.startAuthorization).toHaveBeenCalledOnce();

    const [name, value, options] = cookies.set.mock.calls[0] as [string, string, object];
    expect(name).toBe(authorizationTransactionCookieName);
    expect(readAuthorizationTransaction(value)).toMatchObject({
      codeVerifier: 'pkce-verifier',
      nonce: 'oidc-nonce',
      state: 'oidc-state',
    });
    expect(options).toMatchObject({
      httpOnly: true,
      maxAge: 300,
      path: '/auth',
      sameSite: 'lax',
      secure: true,
    });
  });
});
