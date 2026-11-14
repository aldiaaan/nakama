---
'@nakama/ui': minor
'nakama-desktop': patch
---

Add `AnimatedScrollable` to `@nakama/ui`. It is a scroll area that animates items as they are added and removed, and shows a shadow at the top or bottom edge when more content is past that edge. `AnimatedScrollable.ScrollToTop` is a round button that scales in at the bottom right once the list is scrolled. The Calculations list in the desktop app now uses it.
