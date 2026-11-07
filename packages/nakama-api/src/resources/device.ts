import type { NakamaApi } from '../api';
import { formatBytes, formatPercent } from '../utils/format';

export type CpuCoreInfo = {
  name: string;
  frequencyMhz: number;
  usagePercent: number;
};

export type CpuInfo = {
  brand: string;
  vendorId: string;
  physicalCores: number | null;
  logicalCores: number;
  usagePercent: number;
  cores: CpuCoreInfo[];
};

export type MemoryInfo = {
  totalBytes: number;
  usedBytes: number;
  availableBytes: number;
  freeBytes: number;
  totalSwapBytes: number;
  usedSwapBytes: number;
};

export type GpuInfo = {
  pci: string;
  vendor: string | null;
  model: string | null;
  usagePercent: number | null;
  totalMemoryBytes: number | null;
  usedMemoryBytes: number | null;
};

export type DeviceInfo = {
  cpu: CpuInfo;
  memory: MemoryInfo;
  gpus: GpuInfo[];
  gpuError: string | null;
};

export type CpuCoreView = {
  name: string;
  usagePercent: number;
  usageLabel: string;
  frequencyLabel: string;
};

export type CpuView = {
  brand: string;
  vendor: string;
  physicalCores: string;
  logicalCores: string;
  usagePercent: number;
  usageLabel: string;
  cores: CpuCoreView[];
};

export type MemoryView = {
  summary: string;
  usedPercent: number;
  used: string;
  available: string;
  free: string;
  swap: string;
};

export type GpuView = {
  pci: string;
  title: string;
  detail: string;
  hasUsage: boolean;
  usagePercent: number;
  usageLabel: string;
  vram: string;
};

export type DeviceInfoView = {
  cpu: CpuView;
  memory: MemoryView;
  gpus: GpuView[];
  gpuError: string | null;
};

export class DeviceResource {
  constructor(private readonly api: NakamaApi) {}

  async info(): Promise<DeviceInfoView> {
    const info = await this.api.invoke<DeviceInfo>('device_info');
    return this.format(info);
  }

  private format(info: DeviceInfo): DeviceInfoView {
    return {
      cpu: {
        brand: info.cpu.brand,
        vendor: info.cpu.vendorId || 'Unknown',
        physicalCores: info.cpu.physicalCores?.toString() ?? 'Unknown',
        logicalCores: info.cpu.logicalCores.toString(),
        usagePercent: info.cpu.usagePercent,
        usageLabel: formatPercent(info.cpu.usagePercent),
        cores: info.cpu.cores.map((core) => ({
          name: core.name,
          usagePercent: core.usagePercent,
          usageLabel: formatPercent(core.usagePercent),
          frequencyLabel: `${core.frequencyMhz} MHz`,
        })),
      },
      memory: {
        summary: `${formatBytes(info.memory.usedBytes)} / ${formatBytes(info.memory.totalBytes)}`,
        usedPercent: (info.memory.usedBytes / Math.max(info.memory.totalBytes, 1)) * 100,
        used: formatBytes(info.memory.usedBytes),
        available: formatBytes(info.memory.availableBytes),
        free: formatBytes(info.memory.freeBytes),
        swap: `${formatBytes(info.memory.usedSwapBytes)} / ${formatBytes(info.memory.totalSwapBytes)}`,
      },
      gpus: info.gpus.map((gpu) => ({
        pci: gpu.pci,
        title: gpu.model ?? gpu.vendor ?? 'GPU',
        detail: `${gpu.vendor ?? 'Unknown vendor'} · ${gpu.pci}`,
        hasUsage: gpu.usagePercent != null,
        usagePercent: gpu.usagePercent ?? 0,
        usageLabel: gpu.usagePercent == null ? 'n/a' : formatPercent(gpu.usagePercent),
        vram: this.formatGpuMemory(gpu),
      })),
      gpuError: info.gpuError,
    };
  }

  private formatGpuMemory(gpu: GpuInfo): string {
    const used = gpu.usedMemoryBytes;
    const total = gpu.totalMemoryBytes;
    if (used != null && total != null) {
      return `${formatBytes(used)} / ${formatBytes(total)}`;
    }
    if (total != null) {
      return formatBytes(total);
    }
    return 'Shared or unavailable';
  }
}
