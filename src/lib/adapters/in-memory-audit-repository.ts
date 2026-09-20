import type {
  AuditEntry,
  AuditRepository,
  NewAuditEntry,
} from '../application/audit/audit-repository';

export class InMemoryAuditRepository implements AuditRepository {
  private nextId = 1;
  private readonly entries: AuditEntry[] = [];

  async findAll() {
    return [...this.entries].reverse();
  }

  async record(entry: NewAuditEntry) {
    const stored = { ...entry, id: `audit-${this.nextId++}`, occurredAt: new Date() };

    this.entries.push(stored);

    return stored;
  }
}
