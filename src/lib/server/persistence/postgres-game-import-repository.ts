import type {
  GameImportProvenance,
  GameImportRepository,
} from '../../application/imports/game-import-repository';
import { createDatabase } from './database';
import { gameImportProvenance } from './schema';

type Database = ReturnType<typeof createDatabase>;

export const createPostgresGameImportRepository = (database: Database): GameImportRepository => ({
  async save(provenance: GameImportProvenance) {
    const [stored] = await database
      .insert(gameImportProvenance)
      .values({
        importedAt: provenance.importedAt,
        occurrenceId: provenance.occurrenceId,
        sourceName: provenance.sourceName,
        sourceRow: provenance.sourceRow,
      })
      .returning({
        id: gameImportProvenance.id,
        importedAt: gameImportProvenance.importedAt,
        occurrenceId: gameImportProvenance.occurrenceId,
        sourceName: gameImportProvenance.sourceName,
        sourceRow: gameImportProvenance.sourceRow,
      });

    if (!stored) {
      throw new Error('PostgreSQL did not return the stored game import provenance');
    }

    return stored;
  },
});
