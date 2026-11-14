import { exampleDaemonName, useDaemonOverview } from '@nakama/api';

import { CalculationsSection } from './calculations-section';
import { DaemonStatusSection } from './daemon-status-section';
import { IpcSection } from './ipc-section';
import { MetricsSection } from './metrics-section';

export function ExampleDaemonPanel() {
  const { data: overview, errorMessage, statusLabel } = useDaemonOverview(exampleDaemonName);

  return (
    <section className="flex flex-col gap-6">
      {errorMessage ? <p className="text-destructive text-sm">{errorMessage}</p> : null}
      {statusLabel ? <p className="text-muted-foreground text-sm">{statusLabel}</p> : null}

      {overview ? (
        <>
          <DaemonStatusSection status={overview.status}>
            Four subsystems pick a random seed, run a calculation on a worker thread, and rest for 400 ms. About one
            seed in 40 is rejected, so the daemon restarts that subsystem with backoff.
          </DaemonStatusSection>
          <CalculationsSection />
          <IpcSection name={overview.name} ipc={overview.ipc} />
          <MetricsSection metrics={overview.metrics} />
        </>
      ) : null}
    </section>
  );
}
