# Lighthouse and SEO META Verification

This record audits the built OpenTube web frontend and checks its browser metadata against the published SEO META in 1 CLICK field list. Lighthouse measures this browser run; it is separate from SonarQube analysis and JEV readiness evaluation.

## Contents

- [Scope](#scope)
- [Lighthouse results](#lighthouse-results)
- [Agentic Browsing](#agentic-browsing)
- [SEO META checklist](#seo-meta-checklist)
- [Findings](#findings)
- [Related documentation](#related-documentation)

## Scope

- **Date/time:** 2026-09-25 12:31:59 America/Sao_Paulo.
- **Source:** dirty worktree based on revision e5c1b57.
- **Environment:** Linux; Google Chrome 154.0.8037.57; Lighthouse CLI 13.5.0; default mobile emulation and simulated throttling.
- **Target:** production Vite preview at http://127.0.0.1:4173/; one app preview ran at a time and was stopped before the next.
- **Commands:** `pnpm run build`; `npx --yes lighthouse http://127.0.0.1:4173/ --output=json --output-path=/tmp/lighthouse-opentube.json --only-categories=performance,accessibility,best-practices,seo --chrome-flags='--headless --no-sandbox' --quiet`.
- The video and identity services were not deployed; the public-feed request returned HTTP 500 through the local preview proxy. No live video feed or WebMCP-native browser-agent session was tested.

## Lighthouse results

| Performance | Accessibility | Best practices | SEO |
| ---: | ---: | ---: | ---: |
| 95 | 100 | 96 | 63 |

Scores range from 0 to 100. The SEO score reflects the intentional noindex directive; Best practices is affected by the unavailable API. Lighthouse estimated 188 KiB of unused JavaScript savings on the initial page.

## Agentic Browsing

Lighthouse 13.5.0 on Chrome 154.0.8037.57 ran `npx --yes lighthouse http://127.0.0.1:4173/ --output=json --output-path=/tmp/lighthouse-opentube-agentic.json --only-categories=agentic-browsing --chrome-flags='--headless --no-sandbox' --quiet` at 2026-09-25 12:50:08 America/Sao_Paulo. Its experimental fractional result was 0.50; this is not a 0–100 score. The accessibility-tree and layout-stability audits passed. The root `llms.txt` and `ai-catalog.json` discovery checks failed because no public agent catalog is shipped for this local demo. WebMCP form coverage, registered tools, and schema validity were reported as not applicable: this run did not use the WebMCP origin trial, so it did not verify native registration of `search_public_videos`.

## SEO META checklist

| Field | Result |
| --- | --- |
| HTML language | English is declared. |
| Title and length | Present: “OpenTube — Watch what moves you” (31 characters). |
| Description and length | Present: “OpenTube is a local educational video sharing MVP.” (50 characters). |
| Robots metadata | noindex, nofollow is intentional for this local demo. A valid robots.txt is served with Allow: / so crawlers can read the page directive. |
| Canonical URL | Omitted because the local preview/deployment origin is not a stable public URL. |
| Headings | One H1 is shown on the landing state; the watch state uses its own H1. |
| Images and alt text | No HTML image elements; thumbnails are CSS surfaces and the player is a video element. |
| Links | One labeled internal brand anchor in the empty-feed landing state; one unique target and no external links. |
| Favicon | SVG favicon is declared and served. |
| Open Graph, Twitter, and sitemap | Not provided because the app has no public share URL or public indexing target. |

The SEO META in 1 CLICK extension was not installed in the browser. This is a manual checklist audit against its published fields, not a claim that the extension itself ran.

## Findings

- Lighthouse confirmed the title, description, valid robots.txt, and page metadata. It reports the page as not crawlable by design because of the noindex directive.
- The previously reported mismatch between the visible avatar initials and its accessible name was corrected; final Lighthouse accessibility score is 100.
- Best practices scored 96 because the preview proxy could not reach the video API. The 95 performance score also identifies unused initial JavaScript; consider code splitting the playback dependency if bundle performance becomes a priority.
- `pnpm exec vitest run src/webmcp.test.ts` passed: 2 focused tool tests. `pnpm exec vitest run src/api.test.ts` passed: 11 API-client tests. Lighthouse does not prove native WebMCP browser-agent interoperability.

## Related documentation

- [Verification index](README.md)
- [OpenTube web frontend guide](../../web-frontend/README.md)
- [Observed system results](results.md)
- [JEV readiness assessment](jev-readiness.md)
- [Chrome Lighthouse overview](https://developer.chrome.com/docs/lighthouse/overview)
- [Lighthouse Agentic Browsing scoring](https://developer.chrome.com/docs/lighthouse/agentic-browsing/scoring)
- [SEO META in 1 CLICK listing](https://chromewebstore.google.com/detail/seo-meta-in-1-click/bjogjfinolnhfhkbipphpdlldadpnmhc?hl=en-GB)
