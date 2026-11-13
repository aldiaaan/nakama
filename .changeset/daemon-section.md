---
'nakama-daemon': minor
'@nakama/api': minor
'@nakama/ui': patch
'nakama-desktop': minor
---

Add a Daemon section to the desktop app. An example daemon runs random prime, Monte Carlo, matrix, and Collatz calculations in the background, and the page shows its subsystems, IPC channel, profilers, and metrics. `HeapProfiler::usage` reports live heap numbers, and `Badge` is exported from `@nakama/ui`.
