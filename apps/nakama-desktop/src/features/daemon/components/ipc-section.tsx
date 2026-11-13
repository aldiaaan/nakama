import { ipcRequestChoices, useDaemonIpcRequest, type IpcView } from '@nakama/api';
import { AnimatedNumber, Button, PreferenceCard } from '@nakama/ui/components';

import { Stat } from '../../../shared/components/stat';
import { StatusBadge } from './status-badge';

export type IpcSectionProps = {
  ipc: IpcView;
};

export function IpcSection({ ipc }: IpcSectionProps) {
  const send = useDaemonIpcRequest();

  return (
    <PreferenceCard.Root>
      <div className="flex items-start justify-between gap-4">
        <PreferenceCard.Header>
          <PreferenceCard.Title>IPC</PreferenceCard.Title>
          <PreferenceCard.Description>
            One JSON object per line. The page pings the daemon with a status request every second.
          </PreferenceCard.Description>
        </PreferenceCard.Header>
        <StatusBadge tone={ipc.tone}>{ipc.statusLabel}</StatusBadge>
      </div>
      <PreferenceCard.Content className="flex flex-col gap-4">
        <dl className="grid gap-3">
          <Stat
            label="Endpoint"
            value={<span className="font-mono text-xs break-all select-all">{ipc.endpoint}</span>}
          />
          <Stat label="Round trip" value={<AnimatedNumber>{ipc.roundTrip}</AnimatedNumber>} />
        </dl>
        {ipc.error ? <p className="text-destructive break-words">{ipc.error}</p> : null}
        <div className="flex flex-wrap gap-2">
          {ipcRequestChoices.map((choice) => (
            <Button
              key={choice.value}
              size="sm"
              variant={choice.value === 'shutdown' ? 'destructive' : 'outline'}
              disabled={send.isPending}
              onClick={() => send.mutate(choice.value)}
            >
              {choice.label}
            </Button>
          ))}
        </div>
        {send.data ? (
          <div className="flex flex-col gap-2">
            <p className="text-muted-foreground">{send.data.summary}</p>
            <pre className="bg-muted/60 overflow-x-auto rounded-xl p-3 font-mono text-xs">{send.data.request}</pre>
            <pre className="bg-muted/60 max-h-64 overflow-auto rounded-xl p-3 font-mono text-xs">
              {send.data.response}
            </pre>
          </div>
        ) : null}
      </PreferenceCard.Content>
      {send.statusLabel || send.errorMessage ? (
        <PreferenceCard.Footer>
          {send.statusLabel ? <p className="text-muted-foreground">{send.statusLabel}</p> : null}
          {send.errorMessage ? <PreferenceCard.Error>{send.errorMessage}</PreferenceCard.Error> : null}
        </PreferenceCard.Footer>
      ) : null}
    </PreferenceCard.Root>
  );
}
