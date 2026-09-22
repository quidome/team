import { describe, expect, it } from 'vitest';

import { previewPlayerImport } from './player-import';

describe('player import preview', () => {
  it('parses a fully mapped roster row', () => {
    const preview = previewPlayerImport(
      'first,last,born,assoc,jersey,type\nAvery,Smith,2011-06-15,12345,7,trains and plays',
      {
        associationId: 'assoc',
        birthDate: 'born',
        firstName: 'first',
        jerseyNumber: 'jersey',
        lastName: 'last',
        participationType: 'type',
      },
    );

    expect(preview.issues).toEqual([]);
    expect(preview.records).toEqual([
      {
        associationId: '12345',
        birthDate: '2011-06-15',
        firstName: 'Avery',
        jerseyNumber: 7,
        lastName: 'Smith',
        participationType: 'trains_and_plays',
        sourceRow: 2,
      },
    ]);
  });

  it('requires only a first name and defaults participation type', () => {
    const preview = previewPlayerImport('first\nAvery', { firstName: 'first' });

    expect(preview.issues).toEqual([]);
    expect(preview.records).toEqual([
      { firstName: 'Avery', participationType: 'trains_and_plays', sourceRow: 2 },
    ]);
  });

  it('reports a malformed birthdate and an unrecognized participation type', () => {
    const preview = previewPlayerImport('first,born,type\nAvery,not-a-date,unclear', {
      birthDate: 'born',
      firstName: 'first',
      participationType: 'type',
    });

    expect(preview.issues).toEqual([
      expect.objectContaining({ field: 'birthDate', row: 2 }),
      expect.objectContaining({ field: 'participationType', row: 2 }),
    ]);
    expect(preview.records).toEqual([]);
  });

  it('accepts a tolerant day-first birthdate and trains-only participation', () => {
    const preview = previewPlayerImport('first,born,type\nAvery,15-06-2011,trains_only', {
      birthDate: 'born',
      firstName: 'first',
      participationType: 'type',
    });

    expect(preview.issues).toEqual([]);
    expect(preview.records).toEqual([
      {
        birthDate: '2011-06-15',
        firstName: 'Avery',
        participationType: 'trains_only',
        sourceRow: 2,
      },
    ]);
  });
});
