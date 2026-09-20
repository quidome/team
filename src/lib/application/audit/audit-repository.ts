export type AuditMetadata = Record<string, boolean | number | string>;

export interface AuditEntry {
  action: string;
  entityId: string;
  entityType: string;
  id: string;
  metadata: AuditMetadata;
  occurredAt: Date;
}

export interface NewAuditEntry {
  action: string;
  entityId: string;
  entityType: string;
  metadata: AuditMetadata;
}

export interface AuditRepository {
  findAll(): Promise<AuditEntry[]>;
  record(entry: NewAuditEntry): Promise<AuditEntry>;
}
