import type { NakamaApi } from '../api';
import { formatBytes, formatCount, formatUptime } from '../utils/format';
import type { MetricRowView, StatusTone } from './daemon';

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

/** The app's CPU and heap profilers. They cover the whole app process, so every daemon at once. */
export class ProfilersResource {
  constructor(private readonly api: NakamaApi) {}

  async list(): Promise<ProfilerView[]> {
    const reports = await this.api.invoke<ProfilerReport[]>('profiler_list');
    return reports.map((report) => this.format(report));
  }

  async start(kind: ProfilerKind): Promise<void> {
    await this.api.invoke<null>('profiler_start', { kind });
  }

  async stop(kind: ProfilerKind): Promise<void> {
    await this.api.invoke<null>('profiler_stop', { kind });
  }

  private format(profiler: ProfilerReport): ProfilerView {
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

function blocks(bytes: number, count: number): string {
  return `${formatBytes(bytes)} in ${formatCount(count)} blocks`;
}
