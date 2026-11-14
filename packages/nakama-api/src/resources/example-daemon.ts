import type { NakamaApi } from '../api';
import { formatCount, formatDuration } from '../utils/format';

/** The name the desktop app registers the example daemon under. Pass it to the daemon resource. */
export const exampleDaemonName = 'nakama-example-daemon';

export type CalculationOutcome =
  | { kind: 'primes'; limit: number; count: number; largest: number }
  | { kind: 'monteCarlo'; samples: number; inside: number; estimate: number }
  | { kind: 'matrix'; size: number; trace: number; checksum: number }
  | { kind: 'collatz'; start: number; span: number; longest: number; steps: number };

export type Calculation = {
  id: number;
  subsystem: string;
  seed: number;
  durationUs: number;
  finishedAtMs: number;
  outcome: CalculationOutcome;
};

export type ExampleDaemonReport = {
  jobs: number;
  rejectedInputs: number;
  piEstimate: number | null;
  calculations: Calculation[];
};

export type CalculationView = {
  id: number;
  title: string;
  input: string;
  output: string;
  seed: string;
  duration: string;
  finishedAt: string;
};

export type ExampleDaemonView = {
  jobs: string;
  rejected: string;
  piEstimate: string;
  calculations: CalculationView[];
};

/** What only the example daemon reports. Its state, IPC, and metrics come from the daemon resource. */
export class ExampleDaemonResource {
  constructor(private readonly api: NakamaApi) {}

  async report(): Promise<ExampleDaemonView> {
    const report = await this.api.invoke<ExampleDaemonReport>('example_daemon_report');
    return {
      jobs: formatCount(report.jobs),
      rejected: formatCount(report.rejectedInputs),
      piEstimate: report.piEstimate == null ? '—' : report.piEstimate.toFixed(6),
      calculations: report.calculations.map((calculation) => ({
        id: calculation.id,
        ...describeOutcome(calculation.outcome),
        seed: `seed ${calculation.seed}`,
        duration: formatDuration(calculation.durationUs / 1000),
        finishedAt: new Date(calculation.finishedAtMs).toLocaleTimeString(),
      })),
    };
  }
}

function describeOutcome(outcome: CalculationOutcome): Pick<CalculationView, 'title' | 'input' | 'output'> {
  switch (outcome.kind) {
    case 'primes':
      return {
        title: 'Primes',
        input: `up to ${formatCount(outcome.limit)}`,
        output: `${formatCount(outcome.count)} primes, largest ${formatCount(outcome.largest)}`,
      };
    case 'monteCarlo':
      return {
        title: 'Monte Carlo π',
        input: `${formatCount(outcome.samples)} samples`,
        output: `π ≈ ${outcome.estimate.toFixed(6)}, off by ${Math.abs(outcome.estimate - Math.PI).toExponential(2)}`,
      };
    case 'matrix':
      return {
        title: 'Matrix product',
        input: `${outcome.size} × ${outcome.size}`,
        output: `trace ${outcome.trace.toFixed(3)}, sum ${outcome.checksum.toFixed(3)}`,
      };
    case 'collatz':
      return {
        title: 'Collatz',
        input: `${formatCount(outcome.start)} to ${formatCount(outcome.start + outcome.span - 1)}`,
        output: `${formatCount(outcome.longest)} takes ${formatCount(outcome.steps)} steps`,
      };
  }
}
