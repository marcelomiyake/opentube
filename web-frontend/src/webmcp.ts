import type { WebMCP } from 'webmcp-types'
import type { VideoPage } from './api'

type ListVideos = (query: string, cursor: string | null, limit: number, signal?: AbortSignal) => Promise<VideoPage>

const videoSearchSchema = {
  type: 'object',
  properties: {
    query: {
      type: 'string',
      description: 'A public video search query of up to 200 characters.',
      minLength: 1,
      maxLength: 200,
    },
  },
  required: ['query'],
  additionalProperties: false,
} as const

export interface ToolRegistration {
  ready: Promise<void>
  unregister: () => void
}

export function registerPublicVideoSearchTool(
  modelContext: WebMCP.ModelContext | undefined,
  listVideos: ListVideos,
): ToolRegistration | null {
  if (!modelContext || typeof modelContext.registerTool !== 'function') return null

  const registration = new AbortController()
  const ready = modelContext.registerTool({
    name: 'search_public_videos',
    title: 'Search public videos',
    description: 'Searches ready, public OpenTube videos and returns brief metadata. It does not sign in, upload, play media, or change data.',
    inputSchema: videoSearchSchema,
    annotations: {
      readOnlyHint: true,
      untrustedContentHint: true,
    },
    async execute({ query }, { signal }) {
      if (typeof query !== 'string' || query.trim().length === 0 || query.trim().length > 200) {
        throw new Error('Provide a video search query containing 1 to 200 characters.')
      }
      const page = await listVideos(query.trim(), null, 8, signal)
      const videos = page.items.slice(0, 8).map(({ id, title, creator, published_at }) => ({
        id,
        title,
        creator,
        published_at,
      }))
      return {
        content: [{
          type: 'text',
          text: JSON.stringify({ query: query.trim(), videos, hasMore: Boolean(page.next_cursor) }),
        }],
      }
    },
  }, { signal: registration.signal }).catch(() => {
    registration.abort()
  })

  return {
    ready,
    unregister: () => registration.abort(),
  }
}
