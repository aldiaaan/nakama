import type { CalculationView } from '@nakama/api';
import { PreferenceCard } from '@nakama/ui/components';

export type CalculationsSectionProps = {
  calculations: CalculationView[];
};

export function CalculationsSection({ calculations }: CalculationsSectionProps) {
  return (
    <PreferenceCard.Root>
      <PreferenceCard.Header>
        <PreferenceCard.Title>Calculations</PreferenceCard.Title>
        <PreferenceCard.Description>
          The latest results, newest first. A seed reproduces its input.
        </PreferenceCard.Description>
      </PreferenceCard.Header>
      <PreferenceCard.Content>
        {calculations.length === 0 ? (
          <p className="text-muted-foreground">Waiting for the first result…</p>
        ) : (
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
        )}
      </PreferenceCard.Content>
    </PreferenceCard.Root>
  );
}
