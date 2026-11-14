import type { NakamaApi } from '../api';
import { formatCount, formatDuration, formatUptime } from '../utils/format';

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

export type DaemonOverview = {
  name: string;
  state: DaemonState;
  error: string | null;
  status: StatusReport | null;
  metrics: DaemonMetrics;
  prometheus: string;
  ipc: IpcReport;
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
  uptime: string;
  subsystemsLabel: string;
  error: string | null;
  subsystems: SubsystemView[];
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

export type DaemonOverviewView = {
  name: string;
  status: DaemonStatusView;
  metrics: MetricsView;
  ipc: IpcView;
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

/** Any daemon the app runs, addressed by its name. */
export class DaemonResource {
  constructor(private readonly api: NakamaApi) {}

  async overview(name: string): Promise<DaemonOverviewView> {
    const overview = await this.api.invoke<DaemonOverview>('daemon_overview', { name });
    return {
      name: overview.name,
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
    };
  }

  async start(name: string): Promise<void> {
    await this.api.invoke<null>('daemon_start', { name });
  }

  async sendIpc(name: string, type: IpcRequestType): Promise<IpcExchangeView> {
    const exchange = await this.api.invoke<IpcExchange>('daemon_ipc_request', { name, request: { type } });
    return {
      request: prettyJson(exchange.request),
      response: prettyJson(exchange.response),
      summary: `Answered in ${formatDuration(exchange.roundTripUs / 1000)}`,
    };
  }

  private formatStatus(overview: DaemonOverview): DaemonStatusView {
    const subsystems = overview.status?.subsystems ?? [];
    const running = subsystems.filter((subsystem) => subsystem.state === 'running').length;
    const uptimeMs = overview.state === 'running' ? overview.status?.uptimeMs : null;
    return {
      name: overview.name,
      stateLabel: capitalize(overview.state),
      tone: daemonTones[overview.state],
      canStart: overview.state === 'stopped' || overview.state === 'failed',
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
    return {
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
