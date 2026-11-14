import type { MetricsView } from '@nakama/api';
import { PreferenceCard } from '@nakama/ui/components';

export type MetricsSectionProps = {
  metrics: MetricsView;
};

export function MetricsSection({ metrics }: MetricsSectionProps) {
  return (
    <PreferenceCard.Root>
      <PreferenceCard.Header>
        <PreferenceCard.Title>Metrics</PreferenceCard.Title>
        <PreferenceCard.Description>
          Counters, gauges, and timings the daemon and its subsystems record into one shared registry.
        </PreferenceCard.Description>
      </PreferenceCard.Header>
      <PreferenceCard.Content className="flex flex-col gap-6">
        <div className="grid gap-6 md:grid-cols-2">
          <MetricTable title="Counters" rows={metrics.counters} />
          <MetricTable title="Gauges" rows={metrics.gauges} />
        </div>

        <div className="flex flex-col gap-2">
          <h3 className="font-medium">Timings</h3>
          <table className="w-full text-left">
            <thead className="text-muted-foreground">
              <tr>
                <th className="py-1 font-normal">Name</th>
                <th className="py-1 text-right font-normal">Count</th>
                <th className="py-1 text-right font-normal">Total</th>
                <th className="py-1 text-right font-normal">Average</th>
              </tr>
            </thead>
            <tbody className="font-mono text-xs">
              {metrics.timings.map((timing) => (
                <tr key={timing.name} className="border-border border-t">
                  <td className="py-1.5">{timing.name}</td>
                  <td className="py-1.5 text-right tabular-nums">{timing.count}</td>
                  <td className="py-1.5 text-right tabular-nums">{timing.total}</td>
                  <td className="py-1.5 text-right tabular-nums">{timing.average}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <div className="flex flex-col gap-2">
          <h3 className="font-medium">Prometheus</h3>
          <pre className="bg-muted/60 max-h-72 overflow-auto rounded-xl p-3 font-mono text-xs">
            {metrics.prometheus}
          </pre>
        </div>
      </PreferenceCard.Content>
    </PreferenceCard.Root>
  );
}

type MetricTableProps = {
  title: string;
  rows: MetricsView['counters'];
};

function MetricTable({ title, rows }: MetricTableProps) {
  return (
    <div className="flex flex-col gap-2">
      <h3 className="font-medium">{title}</h3>
      <dl className="font-mono text-xs">
        {rows.map((row) => (
          <div key={row.name} className="border-border flex justify-between gap-4 border-t py-1.5">
            <dt className="truncate">{row.name}</dt>
            <dd className="tabular-nums">{row.value}</dd>
          </div>
        ))}
      </dl>
    </div>
  );
}
