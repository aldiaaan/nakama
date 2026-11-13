const UNITS = ['B', 'KB', 'MB', 'GB', 'TB'] as const;

export function formatBytes(bytes: number): string {
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }
  const digits = unit === 0 ? 0 : 1;
  return `${value.toFixed(digits)} ${UNITS[unit]}`;
}

export function formatPercent(value: number): string {
  return `${value.toFixed(1)}%`;
}

const countFormat = new Intl.NumberFormat('en-US');

export function formatCount(value: number): string {
  return countFormat.format(value);
}

/** Microseconds under 1 ms, milliseconds under 1 s, then seconds. */
export function formatDuration(ms: number): string {
  if (ms < 1) {
    return `${Math.round(ms * 1000)} µs`;
  }
  if (ms < 1000) {
    return `${ms.toFixed(1)} ms`;
  }
  return `${(ms / 1000).toFixed(2)} s`;
}

/** `1h 02m 03s`, `2m 05s`, or `12s`. */
export function formatUptime(ms: number): string {
  const total = Math.floor(ms / 1000);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  if (hours > 0) {
    return `${hours}h ${pad(minutes)}m ${pad(seconds)}s`;
  }
  if (minutes > 0) {
    return `${minutes}m ${pad(seconds)}s`;
  }
  return `${seconds}s`;
}

function pad(value: number): string {
  return value.toString().padStart(2, '0');
}

export function formatApiError(error: unknown): string | null {
  if (error == null) {
    return null;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}
