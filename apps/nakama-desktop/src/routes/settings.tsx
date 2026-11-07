import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/settings')({
  component: SettingsPage,
});

function SettingsPage() {
  return (
    <main className="view-main bg-background no-scrollbar absolute inset-0 overflow-y-auto">
      <div className="mx-auto flex w-full max-w-5xl flex-col gap-6 p-8">
        <header>
          <h1 className="text-2xl font-semibold tracking-tight">Settings</h1>
          <p className="text-muted-foreground text-sm">Preferences for this machine.</p>
        </header>

        <section className="bg-card ring-foreground/10 flex flex-col gap-2 rounded-2xl p-6 text-sm ring-1">
          <h2 className="font-heading text-base font-medium">Appearance</h2>
          <p className="text-muted-foreground">Theme follows the system for now.</p>
        </section>

        <section className="bg-card ring-foreground/10 flex flex-col gap-2 rounded-2xl p-6 text-sm ring-1">
          <h2 className="font-heading text-base font-medium">About</h2>
          <p className="text-muted-foreground">Nakama Desktop</p>
        </section>
      </div>
    </main>
  );
}
