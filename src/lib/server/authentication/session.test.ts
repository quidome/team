import { describe, expect, it } from 'vitest';

import { createSessionToken, readSessionToken } from './session';

const sessionSecret = 'a-secure-session-secret-with-at-least-thirty-two-characters';
const now = new Date('2026-01-01T12:00:00Z');

describe('coordinator sessions', () => {
  it('round-trips a signed session for the authenticated coordinator', () => {
    const token = createSessionToken(
      {
        expiresAt: new Date('2026-01-01T20:00:00Z'),
        subject: 'pocket-id-subject',
      },
      sessionSecret,
    );

    expect(readSessionToken(token, sessionSecret, now)).toEqual({
      expiresAt: new Date('2026-01-01T20:00:00Z'),
      subject: 'pocket-id-subject',
    });
  });

  it('rejects a modified session token', () => {
    const token = createSessionToken(
      {
        expiresAt: new Date('2026-01-01T20:00:00Z'),
        subject: 'pocket-id-subject',
      },
      sessionSecret,
    );

    expect(readSessionToken(`${token}modified`, sessionSecret, now)).toBeUndefined();
  });

  it('rejects an expired session token', () => {
    const token = createSessionToken(
      {
        expiresAt: new Date('2026-01-01T11:59:59Z'),
        subject: 'pocket-id-subject',
      },
      sessionSecret,
    );

    expect(readSessionToken(token, sessionSecret, now)).toBeUndefined();
  });
});
