# Agent instructions

## Do not

- Capture the whole desktop. I'll take the screenshot and send it to you.

## Commits

Use a [Conventional Commits](https://www.conventionalcommits.org/)-style prefix on every commit (for example `feat:`, `fix:`, `chore:`).

If you change code under `packages/*`, `apps/*`, or `crates/*`, include a scope that matches the area you touched (for example `feat(nakama-device-info)`).

## Versions

Changesets is how package versions move. Do not edit a `version` field by hand in `package.json` or in a crate's `[package]` table.

Record a change with `pnpm changeset`. Apply bumps with `pnpm version-packages`. That command updates `package.json`, writes `CHANGELOG.md`, and copies the version into `Cargo.toml`.

Every library crate under `crates/*` has a private `package.json`. `name` and `version` match that crate's `Cargo.toml`. Add this file in the same change as a new crate. `publish` in `Cargo.toml` stays false.

Do not add a `package.json` under `apps/nakama-desktop/src-tauri`. The desktop app package is the version for that shell.

## Comments

Do not comment every line. Add comments only when the code is non-obvious, temporarily hacky, or needs short context for later readers.

## TypeScript code guidelines

- **Exports:** Use a named export. Use a default export only when a library requires it, including a module that only loads through a dynamic import.
- **Barrels:** Do not re-export a folder through an `index.ts`. Import the file that defines the symbol. A package entry, such as `packages/nakama-api/src/index.ts`, is the one exception.
- **Types:** Prefer `type` over `interface`.
- **Composition:** Build a component as compound parts, in the Radix style, whenever it has more than one slot. Export one object and let the caller assemble the parts, such as `Root`, `Header`, `Title`, and `Item`. Do not collapse those slots into props on a single component. A preference is the example:

  ```tsx
  <PreferenceCard.Root>
    <PreferenceCard.Header>
      <PreferenceCard.Title>Appearance</PreferenceCard.Title>
      <PreferenceCard.Description>Saved to config.ini in the app config folder.</PreferenceCard.Description>
    </PreferenceCard.Header>
    <PreferenceCard.Content>
      <PreferenceTile.Group value={theme} onValueChange={selectTheme}>
        <PreferenceTile.Item value="light">
          <PreferenceTile.Indicator />
          <PreferenceTile.Content>
            <PreferenceTile.Title>Light</PreferenceTile.Title>
            <PreferenceTile.Description>Bright surfaces</PreferenceTile.Description>
          </PreferenceTile.Content>
        </PreferenceTile.Item>
      </PreferenceTile.Group>
    </PreferenceCard.Content>
    <PreferenceCard.Footer>
      <PreferenceCard.Error>{errorMessage}</PreferenceCard.Error>
    </PreferenceCard.Footer>
  </PreferenceCard.Root>
  ```

- **Components:** A React component exports its props type and a named function:

  ```tsx
  export type SettingsProviderProps = {
    children: React.ReactNode;
  };

  export function SettingsProvider({ children }: SettingsProviderProps) {
    return <div>{children}</div>;
  }
  ```

- **Progress bars:** Use an enter animation, or once the value is known, animate from `0` to that value instead of jumping instantly.
- **Forms:** Use `react-hook-form` for every form. Do not keep field values, validation, or submit state in `useState`.
- **Async state:** Use React Query (`@tanstack/react-query`) for loading, error, and success. Do not track those with `useState` and `useEffect`. Command hooks stay in `@nakama/api`; an app calls those hooks and renders the result.
- **Overlays:** A modal, menu, dropdown, dialog, popover, or sheet takes its trigger as a child and owns the open state.

  ```tsx
  <RegistrationModal>
    <RegisterButton />
  </RegistrationModal>
  ```

  Compose that trigger with Base UI's `render` prop. Do not control it with `open={state}` unless it is really, really necessary.
- **File layout:** A component, hook, or util used by one feature lives under `apps/<app>/src/features/<feature>/` in `components`, `hooks`, or `utils`. One used by more than one feature in that app lives under `apps/<app>/src/shared/` in the same three folders. A reusable control, hook, or util lives in `@nakama/ui` (`src/components`, `src/hooks`, `src/utils`). A command hook or a formatter for a command result lives in `@nakama/api`. A route file only composes these. Do not put feature UI in `routes/`.
- **Context:** Pass props through components. Use React context only when that prop passes through more than three components.
- **Zustand:** A store is scoped to its provider. Create it with `createStore` from `zustand/vanilla` inside that provider, and pass the `StoreApi` through context. Do not call `create` from `zustand` at module scope. A component outside the provider cannot read the store.

  `useSettingsApi` returns the store and does not subscribe. Use it for a fire-and-forget read or write (`getState`, `setState`). `useSettings` subscribes through a selector and `shallow`. Use it when the component renders store state. Settings is the example:

  ```tsx
  import { useLoadSettings, type SettingsTheme } from '@nakama/api';
  import { createContext, useEffect, useRef, useState } from 'react';
  import { createStore, type StoreApi } from 'zustand/vanilla';

  export type SettingsProviderProps = {
    children: React.ReactNode;
  };

  type SettingsState = {
    theme: SettingsTheme;
    path: string;
  };

  export type SettingsStore = SettingsState;

  export const SettingsContext = createContext<StoreApi<SettingsStore> | null>(null);

  export function SettingsProvider({ children }: SettingsProviderProps) {
    const loaded = useLoadSettings();
    const [store] = useState(() =>
      createStore<SettingsStore>()(() => ({
        theme: 'light',
        path: '',
      }))
    );
    const hydrated = useRef(false);

    useEffect(() => {
      if (!loaded.data || hydrated.current) {
        return;
      }
      hydrated.current = true;
      store.setState({ theme: loaded.data.theme, path: loaded.data.path });
    }, [loaded.data, store]);

    useEffect(() => {
      const apply = () => {
        document.documentElement.classList.toggle('dark', store.getState().theme === 'dark');
      };
      apply();
      return store.subscribe(apply);
    }, [store]);

    return <SettingsContext.Provider value={store}>{children}</SettingsContext.Provider>;
  }
  ```

  ```ts
  import { useContext } from 'react';

  import { SettingsContext } from '../components/settings-provider';

  export function useSettingsApi() {
    const store = useContext(SettingsContext);
    if (!store) {
      throw new Error('Settings context not found');
    }
    return store;
  }
  ```

  ```ts
  import { shallow } from 'zustand/shallow';
  import { useStoreWithEqualityFn } from 'zustand/traditional';

  import type { SettingsStore } from '../components/settings-provider';
  import { useSettingsApi } from './use-settings-api';

  export function useSettings(): SettingsStore;
  export function useSettings<T>(selector: (state: SettingsStore) => T): T;
  export function useSettings<T>(selector?: (state: SettingsStore) => T) {
    const store = useSettingsApi();
    return useStoreWithEqualityFn(store, selector ?? ((state) => state as T), shallow);
  }
  ```
- **Dumb UI:** Components in `packages/nakama-ui/src/components/ui` are dumb. They take props and render. They do not fetch data, call `@nakama/api`, or own feature logic.

## Rust API

`@nakama/api` is resource based. `NakamaApi` is the client. Each area is a resource on that client, such as `device` and `settings`.

```
packages/nakama-api/src/
  api.ts          NakamaApi. The only caller of Tauri invoke.
  errors.ts       CommandError and NakamaError.
  index.ts        Package entry.
  components/     NakamaProvider.
  hooks/          One React Query file per resource.
  resources/      One file per resource.
  utils/          Shared formatters.
```

A resource file owns that area: the raw command types, the view types an app renders, and the resource class. A method calls `this.api.invoke` and returns the formatted view. Do not return the raw command result to an app.

A hook file matches its resource. The hook calls `useNakama()` and then the resource method. It prepares `errorMessage` and any status label. An app calls the hook and renders those strings.

`NakamaProvider` shares one client. The app keeps its own `QueryClientProvider`.

Call Rust commands only through this package. Do not use Tauri `invoke` from an app or another package.

Prepare logic and formatting here. An app renders the strings and numbers this package already prepared. Do not format bytes, percents, labels, or fallbacks in an app, and do not derive those values there, unless it is really, really necessary. The rules for the current commands are in `packages/nakama-api/README.md`.

## Rust errors

Three layers. Do not collapse them into `String` or `anyhow` everywhere.

Each Rust crate has one `src/error.rs`. The crate root re-exports that type. Do not add a second error module, and do not leave an error enum in `lib.rs`.

`crates/*` and other core modules define a public `thiserror` enum in that file. Wrap sources with `#[from]` or `#[source]`. Depend on `thiserror` from the workspace. Do not use `anyhow` here, and do not use a `String` as the error type.

Tauri commands in `apps/nakama-desktop/src-tauri` return `Result<T, CommandError>`. `CommandError` lives in that crate's `src/error.rs`. It derives `thiserror::Error`, `Debug`, and `serde::Serialize`, with `#[serde(tag = "type", content = "message")]`. Implement `From` for each domain error and convert with `?`. Do not return `String` or `anyhow::Error` from a command.

`NakamaApi.invoke` is the only caller of Tauri `invoke`. Its private `safeInvoke` throws `NakamaError`. `message` is the IPC string. `type` is the Rust tag, or `Unknown` when Rust sent a bare string. Apps render the string this package already prepared. They do not read the IPC object.

`main.rs` and other log-and-bail edges return `anyhow::Result` and attach context with `.context("...")`. Do not match variants there. `anyhow` does not get its own module.
