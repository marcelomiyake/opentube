# android-mobile-frontend guidance

> Human guide: [README.md](../README.md) · [Documentation index](../docs/README.md)

- Use Kotlin, Compose, lifecycle-aware state, and Material 3 components.
- Do not store passwords. Keep the local JWT in app-private storage and clear it on sign-out.
- Use a stable cursor and deduplicate rows when the lazy list requests another page.
- Stream upload bytes to the presigned URL; do not load a 1 GB file into memory.
- Test view models and Compose flows; use emulator tests for login, upload, paging, and HLS playback.
