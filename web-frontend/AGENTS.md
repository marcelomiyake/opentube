# web-frontend guidance

> Human guide: [README.md](../README.md) · [Documentation index](../docs/README.md)

- Keep API calls in typed composables and components small and accessible.
- Reset feed cursor and list when the query changes; deduplicate appended IDs.
- Keep an accessible load-more button alongside the intersection sentinel.
- Do not bundle credentials, storage keys, or third-party media. Use synthetic test data.
- Keep the WebMCP tool limited to public video search. Never expose sign-in, upload, playback URLs, or creator-only actions; bound and validate input, mark returned titles/creators as untrusted, forward cancellation, and unregister on teardown.
- Test loading, empty, failure, search, pagination, upload, and playback states.
