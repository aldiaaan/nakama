import { useStartDaemon, type DaemonStatusView } from '@nakama/api';
import { AnimatedNumber, Button, PreferenceCard } from '@nakama/ui/components';

import { Stat } from '../../../shared/components/stat';
import { StatusBadge } from './status-badge';

export type DaemonStatusSectionProps = {
  status: DaemonStatusView;
  /** What the daemon does, shown under its name. */
  children: React.ReactNode;
};

export function DaemonStatusSection({ status, children }: DaemonStatusSectionProps) {
  const start = useStartDaemon(status.name);
  const footer = status.error ?? start.statusLabel ?? start.errorMessage;

  return (
    <PreferenceCard.Root>
      <div className="flex items-start justify-between gap-4">
        <PreferenceCard.Header>
          <PreferenceCard.Title>{status.name}</PreferenceCard.Title>
          <PreferenceCard.Description>{children}</PreferenceCard.Description>
        </PreferenceCard.Header>
        <div className="flex shrink-0 items-center gap-2">
          <StatusBadge tone={status.tone}>{status.stateLabel}</StatusBadge>
          {status.canStart ? (
            <Button size="sm" disabled={start.isPending} onClick={() => start.mutate()}>
              Start
            </Button>
          ) : null}
        </div>
      </div>
      <PreferenceCard.Content className="flex flex-col gap-4">
        <dl className="grid grid-cols-2 gap-3">
          <Stat label="Uptime" value={<AnimatedNumber>{status.uptime}</AnimatedNumber>} />
          <Stat label="Subsystems" value={<AnimatedNumber>{status.subsystemsLabel}</AnimatedNumber>} />
        </dl>
        <ul className="grid gap-3 sm:grid-cols-2">
          {status.subsystems.map((subsystem) => (
            <li key={subsystem.name} className="bg-muted/60 flex flex-col gap-1 rounded-xl px-3 py-2">
              <div className="flex items-center justify-between gap-2">
                <span className="font-mono font-medium">{subsystem.name}</span>
                <StatusBadge tone={subsystem.tone}>{subsystem.stateLabel}</StatusBadge>
              </div>
              <AnimatedNumber className="text-muted-foreground">{subsystem.restartsLabel}</AnimatedNumber>
              {subsystem.lastError ? (
                <p className="text-destructive truncate" title={subsystem.lastError}>
                  {subsystem.lastError}
                </p>
              ) : null}
            </li>
          ))}
        </ul>
      </PreferenceCard.Content>
      {footer ? (
        <PreferenceCard.Footer>
          {status.error ? <PreferenceCard.Error>{status.error}</PreferenceCard.Error> : null}
          {start.statusLabel ? <p className="text-muted-foreground">{start.statusLabel}</p> : null}
          {start.errorMessage ? <PreferenceCard.Error>{start.errorMessage}</PreferenceCard.Error> : null}
        </PreferenceCard.Footer>
      ) : null}
    </PreferenceCard.Root>
  );
}
