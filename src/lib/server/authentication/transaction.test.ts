import { describe, expect, it } from 'vitest';

import { readAuthorizationTransaction, serializeAuthorizationTransaction } from './transaction';

const now = new Date('2026-01-01T12:00:00Z');

describe('OIDC authorization transactions', () => {
  it('round-trips the state, nonce, and PKCE verifier used for a login', () => {
    const cookieValue = serializeAuthorizationTransaction({
      codeVerifier: 'pkce-verifier',
      expiresAt: new Date('2026-01-01T12:05:00Z'),
      nonce: 'oidc-nonce',
      state: 'oidc-state',
    });

    expect(readAuthorizationTransaction(cookieValue, now)).toEqual({
      codeVerifier: 'pkce-verifier',
      expiresAt: new Date('2026-01-01T12:05:00Z'),
      nonce: 'oidc-nonce',
      state: 'oidc-state',
    });
  });

  it('rejects expired transactions', () => {
    const cookieValue = serializeAuthorizationTransaction({
      codeVerifier: 'pkce-verifier',
      expiresAt: new Date('2026-01-01T11:59:59Z'),
      nonce: 'oidc-nonce',
      state: 'oidc-state',
    });

    expect(readAuthorizationTransaction(cookieValue, now)).toBeUndefined();
  });

  it('rejects malformed transactions', () => {
    expect(readAuthorizationTransaction('not-a-transaction', now)).toBeUndefined();
  });
});
