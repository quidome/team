export interface CoordinatorProfile {
  displayName: string;
  email?: string;
}

export interface Coordinator {
  defaultTeamName?: string;
  profile?: CoordinatorProfile;
  subject: string;
  teamNames: string[];
}

export interface CoordinatorRepository {
  findBySubject(subject: string): Promise<Coordinator | undefined>;
  findSubjectsByTeam(teamName: string): Promise<string[]>;
  /** Stores a coordinator for the subject, or returns the existing one. */
  ensure(subject: string): Promise<Coordinator>;
  addTeam(subject: string, teamName: string): Promise<Coordinator>;
  removeTeam(subject: string, teamName: string): Promise<Coordinator>;
  setDefaultTeam(subject: string, teamName: string | undefined): Promise<Coordinator>;
  setProfile(subject: string, profile: CoordinatorProfile): Promise<Coordinator>;
}
