export interface VideoSummary {
  id: string
  title: string
  description: string
  creator: string
  published_at: string
  playback_url: string
}

export interface VideoPage {
  items: VideoSummary[]
  next_cursor: string | null
}

export interface UploadDraft {
  title: string
  description: string
  content_type: 'video/mp4'
  size_bytes: number
}

const base = import.meta.env.VITE_API_BASE ?? '/api'

function responseErrorMessage(path: string, status: number): string {
  if (status === 401) {
    return path === '/v1/login'
      ? 'Email or password did not match the local demo creator account.'
      : 'Your sign-in expired. Sign in again and retry.'
  }
  if (status === 413) return 'File is too large. Choose an MP4 no larger than 1 GB.'
  if (status === 503) return 'OpenTube is temporarily unavailable. Check that the local services are running and retry.'
  return `OpenTube rejected the request (HTTP ${status}). Check the video details and retry.`
}

async function request<T>(path: string, init: RequestInit = {}, token?: string): Promise<T> {
  const headers = new Headers(init.headers)
  if (init.body && !(init.body instanceof FormData)) headers.set('Content-Type', 'application/json')
  if (token) headers.set('Authorization', `Bearer ${token}`)
  const response = await fetch(`${base}${path}`, { ...init, headers })
  if (!response.ok) throw new Error(responseErrorMessage(path, response.status))
  if (response.status === 204) return undefined as T
  return response.json() as Promise<T>
}

export const api = {
  login: (email: string, password: string) => request<{ access_token: string; expires_in: number }>('/v1/login', { method: 'POST', body: JSON.stringify({ email, password }) }),
  listVideos: (query: string, cursor: string | null, limit = 24, signal?: AbortSignal) => {
    const params = new URLSearchParams({ limit: String(limit) })
    if (query.trim()) params.set('query', query.trim())
    if (cursor) params.set('cursor', cursor)
    return request<VideoPage>(`/v1/videos?${params}`, { signal })
  },
  createVideo: (input: UploadDraft, token: string) => request<{ id: string; upload_url: string }>('/v1/videos', { method: 'POST', body: JSON.stringify(input) }, token),
  completeUpload: (id: string, token: string) => request<{ id: string; status: string }>(`/v1/videos/${id}/complete`, { method: 'POST' }, token),
  uploadBytes: async (url: string, file: File) => {
    let response: Response
    try {
      response = await fetch(url, { method: 'PUT', headers: { 'Content-Type': 'video/mp4' }, body: file })
    } catch {
      throw new Error('Could not reach media storage. Make sure the Kind port-forwards are running, then retry.')
    }
    if (!response.ok) {
      throw new Error(`Media storage rejected the upload (HTTP ${response.status}). The upload link may have expired; try again.`)
    }
  }
}

export function appendUniqueVideos(current: VideoSummary[], next: VideoSummary[]): VideoSummary[] {
  const seen = new Set(current.map((video) => video.id))
  const appended: VideoSummary[] = []
  for (const video of next) {
    if (seen.has(video.id)) continue
    seen.add(video.id)
    appended.push(video)
  }
  return [...current, ...appended]
}
