# @nakama/api

Rust commands, their types, and the values an app renders.

Apps import this package. They do not call Tauri `invoke`, and they do not format command results.

## Client

`NakamaApi` is the client. Each area of the API is a resource on that client.

```ts
const api = new NakamaApi();
const info = await api.device.info();
```

`NakamaProvider` shares one client with the tree. `useNakama()` returns that client. A hook that calls `useNakama()` outside `NakamaProvider` throws `useNakama must be used within NakamaProvider`.

```tsx
<NakamaProvider>
    <App />
</NakamaProvider>
```

```tsx
const nakama = useNakama();
await nakama.device.info();
```

The app keeps its own `QueryClientProvider`. `NakamaProvider` only shares the API client.

## Commands

- Keep each command's arguments and raw result types here.
- Add the call as a method on a resource. The resource calls `NakamaApi.invoke`.
- When the command does async work, add a React Query hook in `src/hooks`. The hook calls `useNakama()` and then the resource method.

`api.device.info()` invokes `device_info` and returns `DeviceInfoView`, the formatted snapshot. `useDeviceInfo` polls that command every 500ms and returns the same view. It must run under `NakamaProvider`.

`api.settings.load()` invokes `load_settings` and returns `SettingsView`. `api.settings.save(theme)` invokes `save_settings` and returns the same view. `useLoadSettings` reads the file once. `useSaveSettings` writes it and updates that query. Both must run under `NakamaProvider`.

The daemon resource talks to the example daemon that the desktop app starts in the background. Each daemon hook must run under `NakamaProvider`.

- `api.daemon.overview()` invokes `daemon_overview` and returns `DaemonOverviewView`: the daemon state, its subsystems, metrics, an IPC ping, recent calculations, and profilers. `useDaemonOverview` polls it every second.
- `api.daemon.start()` invokes `daemon_start`.
- `api.daemon.sendIpc(type)` invokes `daemon_ipc_request` and returns `IpcExchangeView`, the request and response JSON lines.
- `api.daemon.startProfiler(kind)` invokes `daemon_start_profiler`. `api.daemon.stopProfiler(kind)` invokes `daemon_stop_profiler`.
- `useStartDaemon`, `useDaemonIpcRequest`, `useStartProfiler`, and `useStopProfiler` wrap those four. Each refreshes the overview once it settles.

## Formatting

Display logic lives here. An app renders the strings and numbers this package already prepared.

`DeviceResource` applies these rules when it formats a snapshot:

- A percent renders with one decimal and a `%` sign. Missing GPU usage renders as `n/a`.
- Bytes use 1024-based units from B through TB. Values under 1 KB stay whole bytes. Larger values use one decimal.
- A missing CPU vendor or physical core count renders as `Unknown`.
- CPU frequency renders as `{mhz} MHz`.
- Memory usage is used bytes divided by total bytes. A total of zero is treated as 1 so the ratio stays finite. The summary and swap lines render as `{used} / {total}`.
- A GPU title is the model, then the vendor, then `GPU`.
- GPU detail renders as `{vendor or "Unknown vendor"} · {pci}`.
- GPU memory renders as `{used} / {total}` when both exist, `{total}` when only the total exists, and `Shared or unavailable` otherwise.
- A query error is a `NakamaError`. Its message is the IPC string, and its type is the Rust tag. A bare string from Rust uses the type `Unknown`. `formatApiError` renders that message. Anything else thrown renders with `String`.

`SettingsResource` applies these rules when it formats settings:

- Theme is `light` or `dark`. The label is `Light` or `Dark`. A theme outside those two renders as the raw value.
- `path` is the `config.ini` path the Rust command already resolved.
- `useLoadSettings` uses `Reading config.ini…` while the file is loading. `useSaveSettings` uses `Saving config.ini…` while the write is in flight.

`DaemonResource` applies these rules when it formats the overview:

- A state label is the state with its first letter capitalized. `tone` is `ok` for running, `busy` while starting, stopping, or restarting, `idle` when stopped, and `error` when failed or aborted. The app picks the color for each tone.
- Uptime renders as `1h 02m 03s`, `2m 05s`, or `12s`, and only while the daemon runs. Otherwise it renders as `—`.
- The lifetime line says whether the daemon runs inside the app or in its own process that outlives it. The process line is `PID {pid}` while the daemon answers, otherwise `—`.
- Counts use `en-US` grouping, such as `1,234,567`.
- Durations render as whole microseconds under 1 ms, as milliseconds with one decimal under 1 s, and as seconds with two decimals after that. A missing IPC round trip renders as `—`.
- A whole gauge renders as a count. Any other gauge renders with three decimals. The π estimate renders with six decimals, or `—` before the first Monte Carlo run.
- A timing's average is its total divided by its count. A count of zero is treated as 1.
- Each calculation gets a title, an input, and an output line for its kind. The finish time uses the locale's time format.
- A profiler that the build cannot run reports why: CPU sampling needs Linux or macOS, or the cargo feature is missing. Heap usage renders as `{bytes} in {count} blocks`.
- IPC request and response lines are pretty-printed JSON. A line that is not JSON renders as is. The summary renders as `Answered in {duration}`.

## Scripts

- `pnpm lint` runs oxlint.
- `pnpm fmt` runs oxfmt.
