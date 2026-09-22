export type DutyType = 'driving' | 'jury' | 'referee';
export type DutySlotStatus = 'assigned' | 'cancelled' | 'completed' | 'incomplete' | 'open';
export type DutySignupStatus = 'selected' | 'volunteer' | 'waitlisted';
export type DutyHistoryStatus =
  'assigned' | 'cancelled' | 'completed' | 'incomplete' | 'reassigned';

export interface DutyRequirements {
  drivingSlots: number;
  jurySlots: number;
  refereeSlots: number;
}

export interface DutySlot {
  assignedPlayerId?: string;
  dutyType: DutyType;
  id: string;
  occurrenceId: string;
  slotNumber: number;
  status: DutySlotStatus;
}

export interface DutySignup {
  dutyType: DutyType;
  occurrenceId: string;
  playerId: string;
  status: DutySignupStatus;
}

export interface DutyAssignmentHistory {
  dutyType: DutyType;
  occurrenceId: string;
  playerId: string;
  slotId: string;
  status: DutyHistoryStatus;
}

export interface DutyFairness {
  completedCount: number;
  playerId: string;
}

export interface DutyView {
  assignmentHistory: DutyAssignmentHistory[];
  fairness: DutyFairness[];
  occurrenceId: string;
  requirements: DutyRequirements;
  signups: DutySignup[];
  slots: DutySlot[];
}

export interface DutyRepository {
  assign(slotId: string, playerId: string): Promise<DutyView>;
  configure(occurrenceId: string, requirements: DutyRequirements): Promise<DutyView>;
  findAllSlots(): Promise<DutySlot[]>;
  findByOccurrence(occurrenceId: string): Promise<DutyView>;
  findFairness(): Promise<DutyFairness[]>;
  recordSignup(signup: DutySignup): Promise<DutyView>;
  updateSlotStatus(slotId: string, status: DutySlotStatus): Promise<DutyView>;
}
