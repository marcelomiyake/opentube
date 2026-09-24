import { expect, test } from '@playwright/test'

const videos = [
  { id: 'one', title: 'Quiet Lake', description: 'Morning light', creator: 'Creator', published_at: '2026-09-20T10:00:00Z', playback_url: 'http://127.0.0.1:19000/opentube/one/master.m3u8' },
  { id: 'two', title: 'Small Observatory', description: 'Stars above', creator: 'Creator', published_at: '2026-09-19T10:00:00Z', playback_url: 'http://127.0.0.1:19000/opentube/two/master.m3u8' },
  { id: 'three', title: 'Night Sky', description: 'A short film', creator: 'Creator', published_at: '2026-09-18T10:00:00Z', playback_url: 'http://127.0.0.1:19000/opentube/three/master.m3u8' },
]

test('loads multiple feed pages without duplicate videos', async ({ page }) => {
  await page.route('**/api/v1/videos*', async (route) => {
    const url = new URL(route.request().url())
    const cursor = url.searchParams.get('cursor')
    await route.fulfill({ json: cursor ? { items: [videos[1], videos[2]], next_cursor: null } : { items: videos.slice(0, 2), next_cursor: 'page-2' } })
  })

  await page.goto('/')
  await expect(page.locator('.video-card')).toHaveCount(2)
  await page.getByRole('button', { name: /load more videos/i }).click()
  await expect(page.locator('.video-card')).toHaveCount(3)
  await expect(page.locator('.card-title')).toHaveText(['Quiet Lake', 'Small Observatory', 'Night Sky'])
})

test('opens the video player and requests its HLS playlist', async ({ page }) => {
  let masterRequested = false
  await page.route('**/api/v1/videos*', (route) => route.fulfill({ json: { items: [videos[2]], next_cursor: null } }))
  await page.route('**/three/master.m3u8', async (route) => {
    masterRequested = true
    await route.fulfill({
      status: 200,
      contentType: 'application/vnd.apple.mpegurl',
      body: '#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1280000,RESOLUTION=640x360\nvariant.m3u8\n',
    })
  })
  await page.route('**/three/variant.m3u8', (route) => route.fulfill({
    status: 200,
    contentType: 'application/vnd.apple.mpegurl',
    body: '#EXTM3U\n#EXT-X-TARGETDURATION:2\n#EXTINF:2,\nsegment.ts\n#EXT-X-ENDLIST\n',
  }))

  await page.goto('/')
  await page.getByRole('button', { name: 'Play Night Sky' }).click()
  await expect(page.getByRole('heading', { name: 'Night Sky' })).toBeVisible()
  await expect(page.getByLabel('Video player')).toBeVisible()
  await expect.poll(() => masterRequested).toBe(true)
})

test('search updates the feed label and requests the matching query', async ({ page }) => {
  let requestedQuery = ''
  await page.route('**/api/v1/videos*', async (route) => {
    requestedQuery = new URL(route.request().url()).searchParams.get('query') ?? ''
    await route.fulfill({ json: { items: requestedQuery ? [videos[2]] : videos.slice(0, 2), next_cursor: null } })
  })

  await page.goto('/')
  await page.getByRole('textbox', { name: 'Search videos' }).fill('night sky')
  await page.getByRole('button', { name: 'Search videos' }).click()
  await expect(page.getByRole('heading', { name: 'Results for “night sky”' })).toBeVisible()
  await expect.poll(() => requestedQuery).toBe('night sky')
})

test('creator can sign in and upload an MP4', async ({ page }) => {
  await page.route('**/api/v1/login', (route) => route.fulfill({ json: { access_token: 'demo-token', token_type: 'Bearer', expires_in: 3600 } }))
  await page.route('**/api/v1/videos**', async (route) => {
    if (route.request().method() === 'POST') {
      await route.fulfill({ status: 201, json: { id: 'new-video', status: 'pending_upload', upload_url: 'https://media.test/put' } })
    } else {
      await route.fulfill({ json: { items: videos.slice(0, 2), next_cursor: null } })
    }
  })
  await page.route('https://media.test/put', (route) => route.fulfill({ status: 200 }))
  await page.route('**/api/v1/videos/new-video/complete', (route) => route.fulfill({ status: 202, json: { id: 'new-video', status: 'uploaded' } }))

  await page.goto('/')
  await page.getByRole('button', { name: /upload/i }).first().click()
  await page.getByLabel('Email').fill('creator@opentube.local')
  await page.getByLabel('Password').fill('synthetic-test-password')
  await page.getByRole('button', { name: 'Sign in' }).last().click()
  await page.getByRole('button', { name: /upload/i }).first().click()
  await page.getByLabel('Title').fill('A short local film')
  await page.locator('.file-picker input[type="file"]').setInputFiles({ name: 'sample.mp4', mimeType: 'video/mp4', buffer: Buffer.from('synthetic video fixture') })
  await page.getByRole('button', { name: 'Upload video' }).click()
  await expect(page.getByRole('status')).toContainText('Upload received')
})
