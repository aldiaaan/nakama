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

## Scripts

- `pnpm lint` runs oxlint.
- `pnpm fmt` runs oxfmt.
