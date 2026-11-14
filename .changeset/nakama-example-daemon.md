---
'nakama-example-daemon': minor
'@nakama/api': minor
'nakama-desktop': minor
---

Move the example daemon out of the desktop app into the `nakama-example-daemon` crate, and let the app run more than one daemon. The crate only defines the daemon: `ExampleDaemon::builder` adds its subsystems, and `ExampleDaemon::report` returns its job counts and calculations. The app hosts each daemon by name, and its profilers now cover the whole process instead of one daemon. In `@nakama/api`, the daemon hooks take a daemon name, and the example daemon and the profilers each have their own resource.
