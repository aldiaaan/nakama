import { useLoadSettings } from '@nakama/api';

import { AppearanceForm } from './appearance-form';

export function SettingsPage() {
  const { errorMessage, statusLabel } = useLoadSettings();

  return (
    <main className="mx-auto flex w-full max-w-5xl flex-col gap-6 p-8">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight">Settings</h1>
        <p className="text-muted-foreground text-sm">Preferences for this machine.</p>
      </header>

      {errorMessage ? <p className="text-destructive text-sm">{errorMessage}</p> : null}

      <section className="bg-card ring-foreground/10 flex flex-col gap-4 rounded-2xl p-6 text-sm ring-1">
        <div>
          <h2 className="font-heading text-base font-medium">Appearance</h2>
          <p className="text-muted-foreground">Saved to config.ini in the app config folder.</p>
        </div>
        {statusLabel ? <p className="text-muted-foreground">{statusLabel}</p> : <AppearanceForm />}
      </section>

      <section className="bg-card ring-foreground/10 flex flex-col gap-2 rounded-2xl p-6 text-sm ring-1">
        <h2 className="font-heading text-base font-medium">About</h2>
        <p className="text-muted-foreground">Nakama Desktop</p>
      </section>
    </main>
  );
}
