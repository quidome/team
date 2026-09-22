import type {
  CoordinatorSettings,
  CoordinatorSettingsRepository,
} from '../application/settings/coordinator-settings-repository';

export class InMemoryCoordinatorSettingsRepository implements CoordinatorSettingsRepository {
  private settings: CoordinatorSettings | undefined;

  constructor(initialSettings?: CoordinatorSettings) {
    this.settings = initialSettings;
  }

  async get(): Promise<CoordinatorSettings | undefined> {
    return this.settings;
  }

  async save(settings: CoordinatorSettings): Promise<CoordinatorSettings> {
    this.settings = settings;

    return settings;
  }
}
