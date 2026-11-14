import { useExampleDaemonReport } from '@nakama/api';
import { AnimatedNumber, PreferenceCard } from '@nakama/ui/components';

import { Stat } from '../../../shared/components/stat';

export function CalculationsSection() {
  const { data: report, errorMessage, statusLabel } = useExampleDaemonReport();
  const calculations = report?.calculations ?? [];

  return (
    <PreferenceCard.Root>
      <PreferenceCard.Header>
        <PreferenceCard.Title>Calculations</PreferenceCard.Title>
        <PreferenceCard.Description>
          The latest results, newest first. A seed reproduces its input.
        </PreferenceCard.Description>
      </PreferenceCard.Header>
      <PreferenceCard.Content className="flex flex-col gap-4">
        {report ? (
          <dl className="grid grid-cols-3 gap-3">
            <Stat label="Jobs" value={<AnimatedNumber>{report.jobs}</AnimatedNumber>} />
            <Stat label="Rejected inputs" value={<AnimatedNumber>{report.rejected}</AnimatedNumber>} />
            <Stat label="Latest π" value={<AnimatedNumber>{report.piEstimate}</AnimatedNumber>} />
          </dl>
        ) : null}
        {report && calculations.length === 0 ? (
          <p className="text-muted-foreground">Waiting for the first result…</p>
        ) : null}
        {calculations.length > 0 ? (
          <ol className="no-scrollbar flex max-h-96 flex-col gap-2 overflow-y-auto">
            {calculations.map((calculation) => (
              <li
                key={calculation.id}
                className="bg-muted/60 grid grid-cols-[1fr_auto] gap-x-4 gap-y-0.5 rounded-xl px-3 py-2"
              >
                <span className="font-medium">
                  {calculation.title} <span className="text-muted-foreground font-normal">{calculation.input}</span>
                </span>
                <span className="text-muted-foreground text-right tabular-nums">{calculation.duration}</span>
                <span className="font-mono text-xs">{calculation.output}</span>
                <span className="text-muted-foreground text-right font-mono text-xs">
                  {calculation.seed} · {calculation.finishedAt}
                </span>
              </li>
            ))}
          </ol>
        ) : null}
      </PreferenceCard.Content>
      {statusLabel || errorMessage ? (
        <PreferenceCard.Footer>
          {statusLabel ? <p className="text-muted-foreground">{statusLabel}</p> : null}
          {errorMessage ? <PreferenceCard.Error>{errorMessage}</PreferenceCard.Error> : null}
        </PreferenceCard.Footer>
      ) : null}
    </PreferenceCard.Root>
  );
}
