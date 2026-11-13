import { useStartProfiler, useStopProfiler, type ProfilerView } from '@nakama/api';
import { AnimateNumber, Button, PreferenceCard } from '@nakama/ui/components';

import { Stat } from '../../../shared/components/stat';
import { StatusBadge } from './status-badge';

export type ProfilersSectionProps = {
  profilers: ProfilerView[];
};

export function ProfilersSection({ profilers }: ProfilersSectionProps) {
  return (
    <PreferenceCard.Root>
      <PreferenceCard.Header>
        <PreferenceCard.Title>Profilers</PreferenceCard.Title>
        <PreferenceCard.Description>
          Profiles land in the profiles folder under the app data folder.
        </PreferenceCard.Description>
      </PreferenceCard.Header>
      <PreferenceCard.Content>
        <ul className="flex flex-col gap-3">
          {profilers.map((profiler) => (
            <ProfilerItem key={profiler.kind} profiler={profiler} />
          ))}
        </ul>
      </PreferenceCard.Content>
    </PreferenceCard.Root>
  );
}

type ProfilerItemProps = {
  profiler: ProfilerView;
};

function ProfilerItem({ profiler }: ProfilerItemProps) {
  const start = useStartProfiler();
  const stop = useStopProfiler();
  const pending = start.isPending || stop.isPending;

  return (
    <li className="bg-muted/60 flex flex-col gap-3 rounded-xl px-3 py-3">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="font-medium">{profiler.title}</p>
          <p className="text-muted-foreground">{profiler.description}</p>
        </div>
        <StatusBadge tone={profiler.tone}>{profiler.statusLabel}</StatusBadge>
      </div>
      {profiler.unavailableReason ? <p className="text-muted-foreground">{profiler.unavailableReason}</p> : null}
      {profiler.heap.length > 0 ? (
        <dl className="grid grid-cols-3 gap-3">
          {profiler.heap.map((row) => (
            <Stat key={row.name} label={row.name} value={<AnimateNumber>{row.value}</AnimateNumber>} />
          ))}
        </dl>
      ) : null}
      {profiler.lastProfile ? (
        <p className="text-muted-foreground font-mono text-xs break-all select-all">{profiler.lastProfile}</p>
      ) : null}
      {profiler.available ? (
        <div>
          {profiler.running ? (
            <Button size="sm" variant="outline" disabled={pending} onClick={() => stop.mutate(profiler.kind)}>
              Stop and save
            </Button>
          ) : (
            <Button size="sm" variant="outline" disabled={pending} onClick={() => start.mutate(profiler.kind)}>
              Start
            </Button>
          )}
        </div>
      ) : null}
      {(start.statusLabel ?? stop.statusLabel) ? (
        <p className="text-muted-foreground">{start.statusLabel ?? stop.statusLabel}</p>
      ) : null}
      {(start.errorMessage ?? stop.errorMessage) ? (
        <PreferenceCard.Error>{start.errorMessage ?? stop.errorMessage}</PreferenceCard.Error>
      ) : null}
    </li>
  );
}
