import { useDaemonOverview } from '@nakama/api';

import { CalculationsSection } from './calculations-section';
import { DaemonStatusSection } from './daemon-status-section';
import { IpcSection } from './ipc-section';
import { MetricsSection } from './metrics-section';
import { ProfilersSection } from './profilers-section';

export function DaemonPage() {
  const { data: overview, errorMessage, statusLabel } = useDaemonOverview();

  return (
    <main className="mx-auto flex w-full max-w-5xl flex-col gap-6 p-8">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight">Daemon</h1>
        <p className="text-muted-foreground text-sm">
          An example daemon that crunches random inputs in the background.
        </p>
      </header>

      {errorMessage ? <p className="text-destructive text-sm">{errorMessage}</p> : null}
      {statusLabel ? <p className="text-muted-foreground text-sm">{statusLabel}</p> : null}

      {overview ? (
        <>
          <DaemonStatusSection status={overview.status} />
          <CalculationsSection calculations={overview.calculations} />
          <div className="grid items-start gap-6 lg:grid-cols-2">
            <IpcSection ipc={overview.ipc} />
            <ProfilersSection profilers={overview.profilers} />
          </div>
          <MetricsSection metrics={overview.metrics} />
        </>
      ) : null}
    </main>
  );
}
