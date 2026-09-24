# android-tv-frontend guidance

> Human guide: [README.md](../README.md) · [Documentation index](../docs/README.md)

- Use Kotlin and AndroidX Compose for TV patterns with predictable D-pad focus.
- Maintain the focused video when more feed items are appended.
- Keep TV browsing and playback usable with a remote; do not add upload controls.
- Use Media3 ExoPlayer for HLS and dispose players with the screen lifecycle.
- Test focus navigation, query reset, pagination, playback errors, and resume behavior.
