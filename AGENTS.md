# Agent instructions

## Do not

- Capture the whole desktop. I'll take the screenshot and send it to you.

## Commits

Use a [Conventional Commits](https://www.conventionalcommits.org/)-style prefix on every commit (for example `feat:`, `fix:`, `chore:`).

If you change code under `packages/*` or `apps/*`, include a scope that matches the area you touched (for example `feat(nakama-ui)`).

## Comments

Do not comment every line. Add comments only when the code is non-obvious, temporarily hacky, or needs short context for later readers.

## UI guidelines

- **Progress bars:** Use an enter animation, or once the value is known, animate from `0` to that value instead of jumping instantly.
