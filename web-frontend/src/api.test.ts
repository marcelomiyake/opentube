import { afterEach, describe, expect, it, vi } from 'vitest'
import { api, appendUniqueVideos, type VideoSummary } from './api'

const video = (id: string): VideoSummary => ({
  id,
  title: `Title ${id}`,
  description: '',
  creator: 'creator',
  published_at: '2026-09-24T00:00:00Z',
  playback_url: 'http://localhost:19000/opentube/video/master.m3u8',
})

afterEach(() => vi.unstubAllGlobals())

describe('API helpers', () => {
  it('appends only new video IDs while preserving order', () => {
    expect(appendUniqueVideos([video('a'), video('b')], [video('b'), video('c')]).map(({ id }) => id)).toEqual(['a', 'b', 'c'])
  })

  it('encodes search and cursor parameters for feed pages', async () => {
    const fetchMock = vi.fn(async (_input: RequestInfo | URL, _init?: RequestInit) => new Response(JSON.stringify({ items: [], next_cursor: null }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)
    await api.listVideos('night sky', 'cursor / 2')
    expect(fetchMock.mock.calls[0]?.[0]).toBe('/api/v1/videos?limit=24&query=night+sky&cursor=cursor+%2F+2')
  })

  it('sends creator credentials and returns the access token', async () => {
    const fetchMock = vi.fn(async (_input: RequestInfo | URL, _init?: RequestInit) => new Response(JSON.stringify({ access_token: 'signed', expires_in: 3600 }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)
    await expect(api.login('creator@opentube.local', 'password')).resolves.toMatchObject({ access_token: 'signed' })
    expect(JSON.parse(String(fetchMock.mock.calls[0]?.[1]?.body))).toEqual({ email: 'creator@opentube.local', password: 'password' })
  })

  it('attaches the bearer token when creating a video', async () => {
    const fetchMock = vi.fn(async (_input: RequestInfo | URL, _init?: RequestInit) => new Response(JSON.stringify({ id: 'new', upload_url: 'https://media.test/put' }), { status: 201 }))
    vi.stubGlobal('fetch', fetchMock)
    await api.createVideo({ title: 'New', description: '', content_type: 'video/mp4', size_bytes: 5 }, 'signed')
    expect(new Headers(fetchMock.mock.calls[0]?.[1]?.headers).get('Authorization')).toBe('Bearer signed')
  })

  it('completes an upload with a bearer token', async () => {
    const fetchMock = vi.fn(async (_input: RequestInfo | URL, _init?: RequestInit) => new Response(JSON.stringify({ id: 'new', status: 'uploaded' }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)
    await api.completeUpload('new', 'signed')
    expect(String(fetchMock.mock.calls[0]?.[0])).toBe('/api/v1/videos/new/complete')
  })

  it('streams MP4 bytes using the presigned PUT URL', async () => {
    const fetchMock = vi.fn(async (_input: RequestInfo | URL, _init?: RequestInit) => new Response(null, { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)
    await api.uploadBytes('https://media.test/put', new File(['video'], 'sample.mp4', { type: 'video/mp4' }))
    expect(fetchMock.mock.calls[0]?.[1]).toMatchObject({ method: 'PUT', headers: { 'Content-Type': 'video/mp4' } })
  })

  it('distinguishes an invalid creator login from an expired session', async () => {
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(new Response('', { status: 401 }))
      .mockResolvedValueOnce(new Response('', { status: 401 }))
    vi.stubGlobal('fetch', fetchMock)

    await expect(api.login('creator', 'wrong')).rejects.toThrow('Email or password did not match')
    await expect(api.listVideos('', null)).rejects.toThrow('Your sign-in expired')
  })

  it('maps service unavailability and other API errors to actionable messages', async () => {
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(new Response('', { status: 503 }))
      .mockResolvedValueOnce(new Response('', { status: 400 }))
    vi.stubGlobal('fetch', fetchMock)

    await expect(api.listVideos('', null)).rejects.toThrow('temporarily unavailable')
    await expect(api.listVideos('', null)).rejects.toThrow('HTTP 400')
  })

  it('maps a too-large upload response to a useful error', async () => {
    vi.stubGlobal('fetch', vi.fn(async (_input: RequestInfo | URL, _init?: RequestInit) => new Response('', { status: 413 })))
    await expect(api.createVideo({ title: 'Large', description: '', content_type: 'video/mp4', size_bytes: 1_073_741_825 }, 'signed')).rejects.toThrow('File is too large.')
  })

  it('explains a network error while sending bytes to media storage', async () => {
    vi.stubGlobal('fetch', vi.fn(async (_input: RequestInfo | URL, _init?: RequestInit) => { throw new TypeError('offline') }))
    await expect(api.uploadBytes('https://media.test/put', new File(['video'], 'sample.mp4', { type: 'video/mp4' }))).rejects.toThrow('Could not reach media storage')
  })

  it('explains an HTTP rejection from media storage', async () => {
    vi.stubGlobal('fetch', vi.fn(async (_input: RequestInfo | URL, _init?: RequestInit) => new Response('', { status: 403 })))
    await expect(api.uploadBytes('https://media.test/put', new File(['video'], 'sample.mp4', { type: 'video/mp4' }))).rejects.toThrow('Media storage rejected the upload (HTTP 403)')
  })
})
