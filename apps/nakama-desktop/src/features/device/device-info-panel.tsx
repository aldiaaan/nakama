import { useDeviceInfo } from '@nakama/api';
import { AnimatedNumber, Button, Progress } from '@nakama/ui/components';

import { Stat } from '../../shared/components/stat';

export function DeviceInfoPanel() {
  const { data: info, errorMessage, isPending, refetch } = useDeviceInfo();

  return (
    <main className="mx-auto flex w-full max-w-5xl flex-col gap-6 p-8">
      <header className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Device</h1>
          <p className="text-muted-foreground text-sm">CPU, memory, and GPU from this machine.</p>
        </div>
        <Button variant="outline" onClick={() => void refetch()}>
          Refresh
        </Button>
      </header>

      {errorMessage ? <p className="text-destructive text-sm">{errorMessage}</p> : null}

      {info ? (
        <div className="grid gap-4">
          <section className="bg-card ring-foreground/10 flex flex-col gap-4 rounded-2xl p-6 text-sm ring-1">
            <div className="flex items-start justify-between gap-4">
              <div>
                <h2 className="font-heading text-base font-medium">CPU</h2>
                <p className="text-muted-foreground">{info.cpu.brand}</p>
              </div>
              <AnimatedNumber className="text-lg font-medium">{info.cpu.usageLabel}</AnimatedNumber>
            </div>
            <Progress value={info.cpu.usagePercent} aria-label="CPU usage" />
            <dl className="grid grid-cols-2 gap-3 sm:grid-cols-4">
              <Stat label="Vendor" value={info.cpu.vendor} />
              <Stat label="Physical cores" value={<AnimatedNumber>{info.cpu.physicalCores}</AnimatedNumber>} />
              <Stat label="Logical cores" value={<AnimatedNumber>{info.cpu.logicalCores}</AnimatedNumber>} />
              <Stat label="Usage" value={<AnimatedNumber>{info.cpu.usageLabel}</AnimatedNumber>} />
            </dl>
            <ul className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
              {info.cpu.cores.map((core) => (
                <li key={core.name} className="bg-muted/60 rounded-xl px-3 py-2">
                  <div className="mb-2 flex items-center justify-between gap-2">
                    <span className="font-medium">{core.name}</span>
                    <AnimatedNumber className="text-muted-foreground">{core.usageLabel}</AnimatedNumber>
                  </div>
                  <Progress value={core.usagePercent} aria-label={`${core.name} usage`} />
                  <AnimatedNumber className="text-muted-foreground mt-2">{core.frequencyLabel}</AnimatedNumber>
                </li>
              ))}
            </ul>
          </section>

          <section className="bg-card ring-foreground/10 flex flex-col gap-4 rounded-2xl p-6 text-sm ring-1">
            <div className="flex items-start justify-between gap-4">
              <h2 className="font-heading text-base font-medium">Memory</h2>
              <AnimatedNumber className="text-muted-foreground">{info.memory.summary}</AnimatedNumber>
            </div>
            <Progress value={info.memory.usedPercent} aria-label="Memory used" />
            <dl className="grid grid-cols-2 gap-3 sm:grid-cols-4">
              <Stat label="Used" value={<AnimatedNumber>{info.memory.used}</AnimatedNumber>} />
              <Stat label="Available" value={<AnimatedNumber>{info.memory.available}</AnimatedNumber>} />
              <Stat label="Free" value={<AnimatedNumber>{info.memory.free}</AnimatedNumber>} />
              <Stat label="Swap" value={<AnimatedNumber>{info.memory.swap}</AnimatedNumber>} />
            </dl>
          </section>

          <section className="bg-card ring-foreground/10 flex flex-col gap-4 rounded-2xl p-6 text-sm ring-1">
            <h2 className="font-heading text-base font-medium">GPU</h2>
            {info.gpuError ? <p className="text-destructive">{info.gpuError}</p> : null}
            {!info.gpuError && info.gpus.length === 0 ? (
              <p className="text-muted-foreground">No GPU reported.</p>
            ) : null}
            <ul className="grid gap-3">
              {info.gpus.map((gpu) => (
                <li key={gpu.pci} className="bg-muted/60 rounded-xl px-3 py-3">
                  <div className="mb-2 flex items-start justify-between gap-3">
                    <div>
                      <p className="font-medium">{gpu.title}</p>
                      <p className="text-muted-foreground">{gpu.detail}</p>
                    </div>
                    <AnimatedNumber>{gpu.usageLabel}</AnimatedNumber>
                  </div>
                  {gpu.hasUsage ? <Progress value={gpu.usagePercent} aria-label={`${gpu.title} usage`} /> : null}
                  <AnimatedNumber className="text-muted-foreground mt-2">{`VRAM ${gpu.vram}`}</AnimatedNumber>
                </li>
              ))}
            </ul>
          </section>
        </div>
      ) : isPending ? (
        <p className="text-muted-foreground text-sm">Reading CPU, memory, and GPU…</p>
      ) : null}
    </main>
  );
}
