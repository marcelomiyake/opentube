import assert from 'node:assert/strict'

const api = process.env.VIDEO_SERVICE_URL ?? 'http://127.0.0.1:18082'
const query = 'pagination-check'
const seen = new Set()
let cursor = null
let pages = 0

do {
  const params = new URLSearchParams({ query, limit: '24' })
  if (cursor) params.set('cursor', cursor)
  const response = await fetch(`${api}/v1/videos?${params}`)
  assert.equal(response.status, 200, 'feed request should succeed')
  const page = await response.json()
  assert.ok(Array.isArray(page.items), 'feed should return items')
  assert.ok(page.items.length <= 24, 'feed must honor the requested limit')
  for (const item of page.items) {
    assert.ok(!seen.has(item.id), `duplicate video in feed: ${item.id}`)
    seen.add(item.id)
  }
  pages += 1
  cursor = page.next_cursor
  assert.ok(pages < 5, 'cursor pagination should terminate')
} while (cursor)

assert.ok(pages >= 2, 'seeded feed should cross a page boundary')
assert.ok(seen.size >= 26, 'all seeded records should be reachable across cursor pages')
console.log(`Cursor pagination passed: ${pages} pages, ${seen.size} distinct videos.`)
