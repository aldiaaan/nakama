import { AppearancePreferenceSection } from './appearance-preference-section';

export function SettingsPage() {
  return (
    <main className="mx-auto flex w-full max-w-5xl flex-col gap-6 p-8">
      <AppearancePreferenceSection />
    </main>
  );
}
