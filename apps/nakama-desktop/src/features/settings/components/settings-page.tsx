import { AnimatedContainerSection } from './animated-container-section';
import { AppearancePreferenceSection } from './appearance-preference-section';
import { MobileSheetSection } from './mobile-sheet-section';

export function SettingsPage() {
  return (
    <main className="mx-auto flex w-full max-w-5xl flex-col gap-6 p-8">
      <AppearancePreferenceSection />
      <MobileSheetSection />
      <AnimatedContainerSection />
    </main>
  );
}
