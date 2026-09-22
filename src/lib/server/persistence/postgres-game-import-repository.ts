import type {
  GameImportProvenance,
  GameImportRepository,
} from '../../application/imports/game-import-repository';
import type { DatabaseConnection } from './database';
import { gameImportProvenance } from './schema';

export const createPostgresGameImportRepository = (
  database: DatabaseConnection,
): GameImportRepository => ({
  async save(provenance: GameImportProvenance) {
    const [stored] = await database
      .insert(gameImportProvenance)
      .values({
        importedAt: provenance.importedAt,
        occurrenceId: provenance.occurrenceId,
        sourceName: provenance.sourceName,
        sourceRow: provenance.sourceRow,
      })
      .onConflictDoUpdate({
        set: {
          importedAt: provenance.importedAt,
          sourceName: provenance.sourceName,
          sourceRow: provenance.sourceRow,
        },
        target: gameImportProvenance.occurrenceId,
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
