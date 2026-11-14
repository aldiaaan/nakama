import { ExampleDaemonPanel } from './example-daemon-panel';
import { ProfilersSection } from './profilers-section';

export function DaemonPage() {
  return (
    <main className="mx-auto flex w-full max-w-5xl flex-col gap-6 p-8">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight">Daemon</h1>
        <p className="text-muted-foreground text-sm">
          Daemons the app runs in the background for as long as it is open, and the profilers that watch them.
        </p>
      </header>

      <ExampleDaemonPanel />
      <ProfilersSection />
    </main>
  );
}
