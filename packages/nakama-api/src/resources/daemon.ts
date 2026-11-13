import type { NakamaApi } from '../api';
import { formatBytes, formatCount, formatDuration, formatUptime } from '../utils/format';

export type DaemonMode = 'embedded' | 'detached';

export type DaemonState = 'starting' | 'running' | 'stopping' | 'stopped' | 'failed';

export type SubsystemState = 'starting' | 'running' | 'restarting' | 'stopping' | 'stopped' | 'failed' | 'aborted';

export type ShutdownReason = 'requested' | { signal: string };

export type SubsystemReport = {
  name: string;
  state: SubsystemState;
  restarts: number;
  lastError: string | null;
};

export type StatusReport = {
  name: string;
  uptimeMs: number | null;
  shutdownReason: ShutdownReason | null;
  subsystems: SubsystemReport[];
};

export type TimingReport = {
  count: number;
  totalMs: number;
};

export type DaemonMetrics = {
  counters: Record<string, number>;
  gauges: Record<string, number>;
  timings: Record<string, TimingReport>;
};

export type IpcReport = {
  endpoint: string;
  reachable: boolean;
  roundTripUs: number | null;
  error: string | null;
};

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

export type ProfilerKind = 'cpu' | 'heap';

export type ProfilerAvailability = 'available' | 'unsupportedPlatform' | 'featureDisabled';

export type HeapUsage = {
  totalBytes: number;
  totalBlocks: number;
  currentBytes: number;
  currentBlocks: number;
  maxBytes: number;
  maxBlocks: number;
};

export type ProfilerReport = {
  kind: ProfilerKind;
  availability: ProfilerAvailability;
  running: boolean;
  elapsedMs: number | null;
  lastProfile: string | null;
  heap: HeapUsage | null;
};

export type DaemonOverview = {
  mode: DaemonMode;
  state: DaemonState;
  error: string | null;
  pid: number | null;
  status: StatusReport | null;
  metrics: DaemonMetrics;
  prometheus: string;
  ipc: IpcReport;
  calculations: Calculation[];
  profilers: ProfilerReport[];
};

export type IpcRequestType = 'status' | 'metrics' | 'shutdown';

export type IpcExchange = {
  request: string;
  response: string;
  roundTripUs: number;
};

/** How a state should read at a glance. The app picks the color. */
export type StatusTone = 'ok' | 'busy' | 'idle' | 'error';

export type SubsystemView = {
  name: string;
  stateLabel: string;
  tone: StatusTone;
  restartsLabel: string;
  lastError: string | null;
};

export type DaemonStatusView = {
  name: string;
  stateLabel: string;
  tone: StatusTone;
  canStart: boolean;
  lifetime: string;
  process: string;
  uptime: string;
  subsystemsLabel: string;
  error: string | null;
  subsystems: SubsystemView[];
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

export type MetricRowView = {
  name: string;
  value: string;
};

export type TimingRowView = {
  name: string;
  count: string;
  total: string;
  average: string;
};

export type MetricsView = {
  jobs: string;
  rejected: string;
  restarts: string;
  piEstimate: string;
  counters: MetricRowView[];
  gauges: MetricRowView[];
  timings: TimingRowView[];
  prometheus: string;
};

export type IpcView = {
  endpoint: string;
  reachable: boolean;
  statusLabel: string;
  tone: StatusTone;
  roundTrip: string;
  error: string | null;
};

export type ProfilerView = {
  kind: ProfilerKind;
  title: string;
  description: string;
  statusLabel: string;
  tone: StatusTone;
  available: boolean;
  running: boolean;
  unavailableReason: string | null;
  lastProfile: string | null;
  heap: MetricRowView[];
};

export type DaemonOverviewView = {
  status: DaemonStatusView;
  metrics: MetricsView;
  ipc: IpcView;
  calculations: CalculationView[];
  profilers: ProfilerView[];
};

export type IpcExchangeView = {
  request: string;
  response: string;
  summary: string;
};

export const ipcRequestChoices: { value: IpcRequestType; label: string }[] = [
  { value: 'status', label: 'Status' },
  { value: 'metrics', label: 'Metrics' },
  { value: 'shutdown', label: 'Shutdown' },
];

const JOBS = 'workload_jobs_total';
const REJECTED = 'workload_rejected_inputs_total';
const RESTARTS = 'daemon_subsystem_restarts_total';
const PI_ESTIMATE = 'workload_pi_estimate';

const lifetimes: Record<DaemonMode, string> = {
  embedded: 'Inside the app, stops when it closes',
  detached: 'Own process, keeps running after the app closes',
};

const daemonTones: Record<DaemonState, StatusTone> = {
  starting: 'busy',
  running: 'ok',
  stopping: 'busy',
  stopped: 'idle',
  failed: 'error',
};

const subsystemTones: Record<SubsystemState, StatusTone> = {
  starting: 'busy',
  running: 'ok',
  restarting: 'busy',
  stopping: 'busy',
  stopped: 'idle',
  failed: 'error',
  aborted: 'error',
};

const profilerCopy: Record<ProfilerKind, { title: string; description: string; feature: string }> = {
  cpu: {
    title: 'CPU profiler',
    description: 'Samples every thread 99 times a second and writes a pprof profile.',
    feature: 'profiling',
  },
  heap: {
    title: 'Heap profiler',
    description: 'Records every allocation and writes a DHAT JSON profile.',
    feature: 'heap-profiling',
  },
};

export class DaemonResource {
  constructor(private readonly api: NakamaApi) {}

  async overview(): Promise<DaemonOverviewView> {
    const overview = await this.api.invoke<DaemonOverview>('daemon_overview');
    return this.format(overview);
  }

  async start(): Promise<void> {
    await this.api.invoke<null>('daemon_start');
  }

  async sendIpc(type: IpcRequestType): Promise<IpcExchangeView> {
    const exchange = await this.api.invoke<IpcExchange>('daemon_ipc_request', { request: { type } });
    return {
      request: prettyJson(exchange.request),
      response: prettyJson(exchange.response),
      summary: `Answered in ${formatDuration(exchange.roundTripUs / 1000)}`,
    };
  }

  async startProfiler(kind: ProfilerKind): Promise<void> {
    await this.api.invoke<null>('daemon_start_profiler', { kind });
  }

  async stopProfiler(kind: ProfilerKind): Promise<void> {
    await this.api.invoke<null>('daemon_stop_profiler', { kind });
  }

  private format(overview: DaemonOverview): DaemonOverviewView {
    return {
      status: this.formatStatus(overview),
      metrics: this.formatMetrics(overview),
      ipc: {
        endpoint: overview.ipc.endpoint,
        reachable: overview.ipc.reachable,
        statusLabel: overview.ipc.reachable ? 'Reachable' : 'Not reachable',
        tone: overview.ipc.reachable ? 'ok' : 'error',
        roundTrip: overview.ipc.roundTripUs == null ? '—' : formatDuration(overview.ipc.roundTripUs / 1000),
        error: overview.ipc.error,
      },
      calculations: overview.calculations.map((calculation) => this.formatCalculation(calculation)),
      profilers: overview.profilers.map((profiler) => this.formatProfiler(profiler)),
    };
  }

  private formatStatus(overview: DaemonOverview): DaemonStatusView {
    const subsystems = overview.status?.subsystems ?? [];
    const running = subsystems.filter((subsystem) => subsystem.state === 'running').length;
    const uptimeMs = overview.state === 'running' ? overview.status?.uptimeMs : null;
    return {
      name: overview.status?.name ?? 'nakama-daemon',
      stateLabel: capitalize(overview.state),
      tone: daemonTones[overview.state],
      canStart: overview.state === 'stopped' || overview.state === 'failed',
      lifetime: lifetimes[overview.mode],
      process: overview.pid == null ? '—' : `PID ${overview.pid}`,
      uptime: uptimeMs == null ? '—' : formatUptime(uptimeMs),
      subsystemsLabel: `${running} of ${subsystems.length} running`,
      error: overview.error,
      subsystems: subsystems.map((subsystem) => ({
        name: subsystem.name,
        stateLabel: capitalize(subsystem.state),
        tone: subsystemTones[subsystem.state],
        restartsLabel: subsystem.restarts === 1 ? '1 restart' : `${formatCount(subsystem.restarts)} restarts`,
        lastError: subsystem.lastError,
      })),
    };
  }

  private formatMetrics({ metrics, prometheus }: DaemonOverview): MetricsView {
    const pi = metrics.gauges[PI_ESTIMATE];
    return {
      jobs: formatCount(metrics.counters[JOBS] ?? 0),
      rejected: formatCount(metrics.counters[REJECTED] ?? 0),
      restarts: formatCount(metrics.counters[RESTARTS] ?? 0),
      piEstimate: pi == null ? '—' : pi.toFixed(6),
      counters: Object.entries(metrics.counters).map(([name, value]) => ({ name, value: formatCount(value) })),
      gauges: Object.entries(metrics.gauges).map(([name, value]) => ({
        name,
        value: Number.isInteger(value) ? formatCount(value) : value.toFixed(3),
      })),
      timings: Object.entries(metrics.timings).map(([name, timing]) => ({
        name,
        count: formatCount(timing.count),
        total: formatDuration(timing.totalMs),
        average: formatDuration(timing.totalMs / Math.max(timing.count, 1)),
      })),
      prometheus: prometheus.trimEnd(),
    };
  }

  private formatCalculation(calculation: Calculation): CalculationView {
    return {
      id: calculation.id,
      ...describeOutcome(calculation.outcome),
      seed: `seed ${calculation.seed}`,
      duration: formatDuration(calculation.durationUs / 1000),
      finishedAt: new Date(calculation.finishedAtMs).toLocaleTimeString(),
    };
  }

  private formatProfiler(profiler: ProfilerReport): ProfilerView {
    const copy = profilerCopy[profiler.kind];
    const available = profiler.availability === 'available';
    let statusLabel = 'Idle';
    if (!available) {
      statusLabel = 'Unavailable';
    } else if (profiler.running) {
      statusLabel = `Recording · ${formatUptime(profiler.elapsedMs ?? 0)}`;
    }
    let unavailableReason: string | null = null;
    if (profiler.availability === 'unsupportedPlatform') {
      unavailableReason = 'CPU sampling needs Linux or macOS.';
    } else if (profiler.availability === 'featureDisabled') {
      unavailableReason = `Run the app with --features ${copy.feature} to turn it on.`;
    }
    const heap = profiler.heap;
    return {
      kind: profiler.kind,
      title: copy.title,
      description: copy.description,
      statusLabel,
      tone: profiler.running ? 'busy' : available ? 'idle' : 'error',
      available,
      running: profiler.running,
      unavailableReason,
      lastProfile: profiler.lastProfile,
      heap: heap
        ? [
            { name: 'Live', value: blocks(heap.currentBytes, heap.currentBlocks) },
            { name: 'Peak', value: blocks(heap.maxBytes, heap.maxBlocks) },
            { name: 'Allocated', value: blocks(heap.totalBytes, heap.totalBlocks) },
          ]
        : [],
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

function blocks(bytes: number, count: number): string {
  return `${formatBytes(bytes)} in ${formatCount(count)} blocks`;
}

function capitalize(value: string): string {
  return value.charAt(0).toUpperCase() + value.slice(1);
}

function prettyJson(raw: string): string {
  try {
    return JSON.stringify(JSON.parse(raw), null, 2);
  } catch {
    return raw;
  }
}
