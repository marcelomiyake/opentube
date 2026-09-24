import type { WebMCP } from 'webmcp-types'
import { describe, expect, it, vi } from 'vitest'
import type { VideoPage } from './api'
import { registerPublicVideoSearchTool } from './webmcp'

function modelContext() {
  const tools: WebMCP.ModelContextTool[] = []
  let registrationSignal: AbortSignal | undefined
  const registerTool = vi.fn(async (tool: WebMCP.ModelContextTool, options?: WebMCP.ModelContextRegisterToolOptions) => {
    tools.push(tool)
    registrationSignal = options?.signal
  })
  return {
    tools,
    get registrationSignal() { return registrationSignal },
    context: { registerTool } as unknown as WebMCP.ModelContext,
  }
}

describe('WebMCP public video search tool', () => {
  it('registers a read-only search and returns bounded public metadata without media URLs', async () => {
    const browser = modelContext()
    const page: VideoPage = {
      items: Array.from({ length: 10 }, (_, index) => ({
        id: `video-${index}`,
        title: `Title ${index}`,
        description: 'untrusted description',
        creator: 'demo creator',
        published_at: '2026-09-25T00:00:00Z',
        playback_url: 'http://media.local/video.m3u8',
      })),
      next_cursor: 'next-page',
    }
    const listVideos = vi.fn(async (_query: string, _cursor: string | null, _limit: number, _signal?: AbortSignal) => page)
    const registration = registerPublicVideoSearchTool(browser.context, listVideos)
    expect(registration).not.toBeNull()
    await registration?.ready
    expect(browser.tools[0]?.name).toBe('search_public_videos')
    expect(browser.tools[0]?.annotations).toEqual({ readOnlyHint: true, untrustedContentHint: true })

    const signal = new AbortController().signal
    const result = await browser.tools[0]!.execute({ query: 'night sky' }, { signal }) as {
      content: Array<{ text: string }>
    }
    expect(listVideos).toHaveBeenCalledWith('night sky', null, 8, signal)
    const output = JSON.parse(result.content[0]!.text)
    expect(output.videos).toHaveLength(8)
    expect(output.hasMore).toBe(true)
    expect(result.content[0]!.text).not.toContain('playback_url')
    expect(result.content[0]!.text).not.toContain('media.local')

    registration?.unregister()
    expect(browser.registrationSignal?.aborted).toBe(true)
  })

  it('rejects empty and overlong queries and remains unavailable without browser support', async () => {
    const listVideos = vi.fn(async () => ({ items: [], next_cursor: null }))
    expect(registerPublicVideoSearchTool(undefined, listVideos)).toBeNull()

    const browser = modelContext()
    const registration = registerPublicVideoSearchTool(browser.context, listVideos)
    await registration?.ready
    await expect(browser.tools[0]!.execute({ query: '  ' }, { signal: new AbortController().signal }))
      .rejects.toThrow('1 to 200 characters')
    await expect(browser.tools[0]!.execute({ query: 'x'.repeat(201) }, { signal: new AbortController().signal }))
      .rejects.toThrow('1 to 200 characters')
    expect(listVideos).not.toHaveBeenCalled()
  })
})
