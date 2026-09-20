export interface GameImportProvenance {
  importedAt: Date;
  occurrenceId: string;
  sourceName: string;
  sourceRow: number;
}

export interface GameImportRepository {
  save(provenance: GameImportProvenance): Promise<GameImportProvenance & { id: string }>;
}
