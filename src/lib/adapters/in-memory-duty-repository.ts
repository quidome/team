import type {
  DutyAssignmentHistory,
  DutyFairness,
  DutyRepository,
  DutyRequirements,
  DutySignup,
  DutySlot,
  DutySlotStatus,
  DutyType,
  DutyView,
} from '../application/duties/duty-repository';

const dutyTypes: DutyType[] = ['referee', 'jury', 'driving'];

const signupKey = (signup: DutySignup) =>
  [signup.occurrenceId, signup.dutyType, signup.playerAssociationId].join('|');

export class InMemoryDutyRepository implements DutyRepository {
  private readonly histories: DutyAssignmentHistory[] = [];
  private readonly requirements = new Map<string, DutyRequirements>();
  private readonly signups = new Map<string, DutySignup>();
  private readonly slots = new Map<string, DutySlot>();
  private nextSlotId = 1;

  async configure(occurrenceId: string, requirements: DutyRequirements): Promise<DutyView> {
    this.requirements.set(occurrenceId, requirements);

    for (const dutyType of dutyTypes) {
      const requiredSlots = requirements[`${dutyType}Slots`];
      const existingSlots = [...this.slots.values()]
        .filter((slot) => slot.occurrenceId === occurrenceId && slot.dutyType === dutyType)
        .sort((left, right) => left.slotNumber - right.slotNumber);

      for (let slotNumber = 1; slotNumber <= requiredSlots; slotNumber += 1) {
        const existingSlot = existingSlots.find((slot) => slot.slotNumber === slotNumber);

        if (existingSlot) {
          if (existingSlot.status === 'cancelled') {
            existingSlot.status = existingSlot.assignedPlayerAssociationId ? 'assigned' : 'open';
          }
          continue;
        }

        const slot: DutySlot = {
          dutyType,
          id: `duty-slot-${this.nextSlotId++}`,
          occurrenceId,
          slotNumber,
          status: 'open',
        };
        this.slots.set(slot.id, slot);
      }

      for (const slot of existingSlots.filter((slot) => slot.slotNumber > requiredSlots)) {
        if (slot.status !== 'cancelled') {
          slot.status = 'cancelled';
          this.addHistory(slot, slot.assignedPlayerAssociationId, 'cancelled');
        }
      }
    }

    return this.findByOccurrence(occurrenceId);
  }

  async findAllSlots(): Promise<DutySlot[]> {
    return [...this.slots.values()].map((slot) => ({ ...slot }));
  }

  async findByOccurrence(occurrenceId: string): Promise<DutyView> {
    const requirements = this.requirements.get(occurrenceId) ?? {
      drivingSlots: 0,
      jurySlots: 0,
      refereeSlots: 0,
    };

    return {
      assignmentHistory: this.histories
        .filter((history) => history.occurrenceId === occurrenceId)
        .map((history) => ({ ...history })),
      fairness: await this.findFairness(),
      occurrenceId,
      requirements: { ...requirements },
      signups: [...this.signups.values()]
        .filter((signup) => signup.occurrenceId === occurrenceId)
        .map((signup) => ({ ...signup })),
      slots: [...this.slots.values()]
        .filter((slot) => slot.occurrenceId === occurrenceId)
        .map((slot) => ({ ...slot })),
    };
  }

  async findFairness(): Promise<DutyFairness[]> {
    const completed = new Map<string, number>();

    for (const history of this.histories.filter((history) => history.status === 'completed')) {
      completed.set(
        history.playerAssociationId,
        (completed.get(history.playerAssociationId) ?? 0) + 1,
      );
    }

    return [...completed.entries()]
      .map(([playerAssociationId, completedCount]) => ({
        completedCount,
        playerAssociationId,
      }))
      .sort((left, right) => left.completedCount - right.completedCount);
  }

  async recordSignup(signup: DutySignup): Promise<DutyView> {
    this.signups.set(signupKey(signup), { ...signup });

    return this.findByOccurrence(signup.occurrenceId);
  }

  async assign(slotId: string, playerAssociationId: string): Promise<DutyView> {
    const slot = this.slots.get(slotId);

    if (!slot) {
      throw new Error(`Duty slot ${slotId} does not exist`);
    }

    if (slot.status === 'cancelled' || slot.status === 'completed') {
      throw new Error(`Duty slot ${slotId} cannot be assigned`);
    }

    if (slot.assignedPlayerAssociationId === playerAssociationId) {
      return this.findByOccurrence(slot.occurrenceId);
    }

    if (slot.assignedPlayerAssociationId) {
      this.addHistory(slot, slot.assignedPlayerAssociationId, 'reassigned');
      this.updateSignupStatus(slot, slot.assignedPlayerAssociationId, 'volunteer');
    }

    slot.assignedPlayerAssociationId = playerAssociationId;
    slot.status = 'assigned';
    this.updateSignupStatus(slot, playerAssociationId, 'selected');
    this.addHistory(slot, playerAssociationId, 'assigned');

    return this.findByOccurrence(slot.occurrenceId);
  }

  async updateSlotStatus(slotId: string, status: DutySlotStatus): Promise<DutyView> {
    const slot = this.slots.get(slotId);

    if (!slot) {
      throw new Error(`Duty slot ${slotId} does not exist`);
    }

    slot.status = status;

    if (slot.assignedPlayerAssociationId && status !== 'assigned' && status !== 'open') {
      this.addHistory(slot, slot.assignedPlayerAssociationId, status);
    }

    return this.findByOccurrence(slot.occurrenceId);
  }

  private addHistory(
    slot: DutySlot,
    playerAssociationId: string | undefined,
    status: DutyAssignmentHistory['status'],
  ): void {
    if (!playerAssociationId) {
      return;
    }

    this.histories.push({
      dutyType: slot.dutyType,
      occurrenceId: slot.occurrenceId,
      playerAssociationId,
      slotId: slot.id,
      status,
    });
  }

  private updateSignupStatus(
    slot: DutySlot,
    playerAssociationId: string,
    status: DutySignup['status'],
  ): void {
    const key = signupKey({
      dutyType: slot.dutyType,
      occurrenceId: slot.occurrenceId,
      playerAssociationId,
      status,
    });
    const existing = this.signups.get(key);

    if (existing) {
      existing.status = status;
      return;
    }

    this.signups.set(key, {
      dutyType: slot.dutyType,
      occurrenceId: slot.occurrenceId,
      playerAssociationId,
      status,
    });
  }
}
