import type { NakamaApi } from '../api';

export type SettingsTheme = 'light' | 'dark';

export type Settings = {
  theme: SettingsTheme;
  path: string;
};

export type SettingsView = {
  theme: SettingsTheme;
  themeLabel: string;
  path: string;
};

export type ThemeChoice = {
  value: SettingsTheme;
  label: string;
  description: string;
};

export const themeChoices = [
  { value: 'light', label: 'Light', description: 'Bright surfaces' },
  { value: 'dark', label: 'Dark', description: 'Dark surfaces' },
] as const satisfies readonly ThemeChoice[];

export class SettingsResource {
  constructor(private readonly api: NakamaApi) {}

  async load(): Promise<SettingsView> {
    const settings = await this.api.invoke<Settings>('load_settings');
    return this.format(settings);
  }

  async save(theme: SettingsTheme): Promise<SettingsView> {
    const settings = await this.api.invoke<Settings>('save_settings', { theme });
    return this.format(settings);
  }

  private format(settings: Settings): SettingsView {
    return {
      theme: settings.theme,
      themeLabel: themeChoices.find((choice) => choice.value === settings.theme)?.label ?? settings.theme,
      path: settings.path,
    };
  }
}
