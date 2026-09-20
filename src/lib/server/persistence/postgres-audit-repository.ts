import { desc } from 'drizzle-orm';

import type {
  AuditEntry,
  AuditRepository,
  NewAuditEntry,
} from '../../application/audit/audit-repository';
import type { DatabaseConnection } from './database';
import { auditEntries } from './schema';

const toAuditEntry = (entry: typeof auditEntries.$inferSelect): AuditEntry => ({
  action: entry.action,
  entityId: entry.entityId,
  entityType: entry.entityType,
  id: entry.id,
  metadata: entry.metadata,
  occurredAt: entry.occurredAt,
});

export const createPostgresAuditRepository = (database: DatabaseConnection): AuditRepository => ({
  async findAll() {
    const rows = await database.select().from(auditEntries).orderBy(desc(auditEntries.occurredAt));

    return rows.map(toAuditEntry);
  },

  async record(entry: NewAuditEntry) {
    const [stored] = await database
      .insert(auditEntries)
      .values({
        action: entry.action,
        entityId: entry.entityId,
        entityType: entry.entityType,
        metadata: entry.metadata,
      })
      .returning();

    if (!stored) {
      throw new Error('PostgreSQL did not return the stored audit entry');
    }

    return toAuditEntry(stored);
  },
});
