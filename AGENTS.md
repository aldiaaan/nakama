# Agent instructions

## Commits

Use a [Conventional Commits](https://www.conventionalcommits.org/)-style prefix on every commit (for example `feat:`, `fix:`, `chore:`).

If you change code under `packages/*` or `apps/*`, include a scope that matches the area you touched (for example `feat(nakama-ui)`).

## License header

Rust, TypeScript, JavaScript, Python, and React source files must start with this license header (make sure you added the empty blank line after header):

```
// nakama
// Copyright (C) 2026 aldiaaan <62921668+aldiaaan@users.noreply.github.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

```

## Comments

Do not comment every line. Add comments only when the code is non-obvious, temporarily hacky, or needs short context for later readers.

## UI guidelines

- **Progress bars:** Use an enter animation, or once the value is known, animate from `0` to that value instead of jumping instantly.
