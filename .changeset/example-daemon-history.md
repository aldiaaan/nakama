---
'nakama-example-daemon': patch
'@nakama/ui': patch
---

The example daemon keeps its latest 200 calculations instead of 24. `AnimatedScrollable` holds the item you are reading in place when items are added above it, unless you are scrolled to the top.
