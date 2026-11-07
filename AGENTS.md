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

## UI guidelines

- **Progress bars:** Use an enter animation, or once the value is known, animate from `0` to that value instead of jumping instantly.

## Rust API

Call Rust commands only through `@nakama/api`. Do not use Tauri `invoke` from an app or another package.

Keep each command's argument and result types in `@nakama/api`.

Add each command as a resource method on `NakamaApi`. When a command does async work, add a React Query hook in `packages/nakama-api/src/hooks`. The hook calls `useNakama()` and then that resource method.

Format command results in `@nakama/api`. Apps render the strings and numbers that package already prepared. Do not format bytes, percents, labels, fallbacks, or other display logic in an app. The rules for the current commands are in `packages/nakama-api/README.md`.

## Rust errors

Three layers. Do not collapse them into `String` or `anyhow` everywhere.

Each Rust crate has one `src/error.rs`. The crate root re-exports that type. Do not add a second error module, and do not leave an error enum in `lib.rs`.

`crates/*` and other core modules define a public `thiserror` enum in that file. Wrap sources with `#[from]` or `#[source]`. Depend on `thiserror` from the workspace. Do not use `anyhow` here, and do not use a `String` as the error type.

Tauri commands in `apps/nakama-desktop/src-tauri` return `Result<T, CommandError>`. `CommandError` lives in that crate's `src/error.rs`. It derives `thiserror::Error`, `Debug`, and `serde::Serialize`, with `#[serde(tag = "type", content = "message")]`. Implement `From` for each domain error and convert with `?`. Do not return `String` or `anyhow::Error` from a command.

`NakamaApi.invoke` is the only caller of Tauri `invoke`. Its private `safeInvoke` throws `NakamaError`. `message` is the IPC string. `type` is the Rust tag, or `Unknown` when Rust sent a bare string. Apps render the string this package already prepared. They do not read the IPC object.

`main.rs` and other log-and-bail edges return `anyhow::Result` and attach context with `.context("...")`. Do not match variants there. `anyhow` does not get its own module.
