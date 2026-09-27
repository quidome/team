import type {
  Coordinator,
  CoordinatorProfile,
  CoordinatorRepository,
} from '../application/coordinators/coordinator-repository';

export class InMemoryCoordinatorRepository implements CoordinatorRepository {
  private readonly coordinators = new Map<string, Coordinator>();

  constructor(initialCoordinators: Coordinator[] = []) {
    for (const coordinator of initialCoordinators) {
      this.coordinators.set(coordinator.subject, coordinator);
    }
  }

  async findBySubject(subject: string): Promise<Coordinator | undefined> {
    return this.coordinators.get(subject);
  }

  async findSubjectsByTeam(teamName: string): Promise<string[]> {
    return [...this.coordinators.values()]
      .filter((coordinator) => coordinator.teamNames.includes(teamName))
      .map((coordinator) => coordinator.subject)
      .sort((left, right) => left.localeCompare(right));
  }

  async ensure(subject: string): Promise<Coordinator> {
    const existing = this.coordinators.get(subject);

    if (existing) {
      return existing;
    }

    const coordinator: Coordinator = { subject, teamNames: [] };
    this.coordinators.set(subject, coordinator);

    return coordinator;
  }

  async addTeam(subject: string, teamName: string): Promise<Coordinator> {
    const coordinator = await this.ensure(subject);
    const teamNames = [...new Set([...coordinator.teamNames, teamName])].sort((left, right) =>
      left.localeCompare(right),
    );

    return this.store({ ...coordinator, teamNames });
  }

  async removeTeam(subject: string, teamName: string): Promise<Coordinator> {
    const coordinator = await this.ensure(subject);

    return this.store({
      ...coordinator,
      teamNames: coordinator.teamNames.filter((name) => name !== teamName),
    });
  }

  async setDefaultTeam(subject: string, teamName: string | undefined): Promise<Coordinator> {
    const { profile, subject: storedSubject, teamNames } = await this.ensure(subject);
    const coordinator: Coordinator = profile
      ? { profile, subject: storedSubject, teamNames }
      : { subject: storedSubject, teamNames };

    return this.store(
      teamName === undefined ? coordinator : { ...coordinator, defaultTeamName: teamName },
    );
  }

  async setProfile(subject: string, profile: CoordinatorProfile): Promise<Coordinator> {
    return this.store({ ...(await this.ensure(subject)), profile });
  }

  private store(coordinator: Coordinator): Coordinator {
    this.coordinators.set(coordinator.subject, coordinator);

    return coordinator;
  }
}
