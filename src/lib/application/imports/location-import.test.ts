import { describe, expect, it } from 'vitest';

import { previewLocationImport } from './location-import';

describe('location import preview', () => {
  it('parses mapped locations and defaults missing travel minutes to zero', () => {
    const preview = previewLocationImport('name,travel\nHome court,20\nAway court,', {
      name: 'name',
      travelMinutes: 'travel',
    });

    expect(preview.issues).toEqual([]);
    expect(preview.records).toEqual([
      { name: 'Home court', sourceRow: 2, travelMinutes: 20 },
      { name: 'Away court', sourceRow: 3, travelMinutes: 0 },
    ]);
  });

  it('reports an unmapped required field', () => {
    const preview = previewLocationImport('name,travel\nHome court,20', {
      travelMinutes: 'travel',
    });

    expect(preview.issues).toEqual([
      expect.objectContaining({ field: 'name', row: 1 }),
      expect.objectContaining({ field: 'name', row: 2 }),
    ]);
  });

  it('rejects a negative or non-integer travel minutes value', () => {
    const preview = previewLocationImport('name,travel\nHome court,-5\nAway court,2.5', {
      name: 'name',
      travelMinutes: 'travel',
    });

    expect(preview.issues).toEqual([
      expect.objectContaining({ field: 'travelMinutes', row: 2 }),
      expect.objectContaining({ field: 'travelMinutes', row: 3 }),
    ]);
    expect(preview.records).toEqual([]);
  });

  it('requires a value for the name column', () => {
    const preview = previewLocationImport('name,travel\n,20', {
      name: 'name',
      travelMinutes: 'travel',
    });

    expect(preview.issues).toEqual([expect.objectContaining({ field: 'name', row: 2 })]);
  });
});
