import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import App from './App.vue'

const first = { id: 'video-1', title: 'Quiet Lake', description: 'A calm morning', creator: 'Creator', published_at: '2026-09-20T10:00:00Z', playback_url: 'http://media.test/master.m3u8' }
const second = { ...first, id: 'video-2', title: 'Small Observatory' }
const third = { ...first, id: 'video-3', title: 'Night Sky' }
const apiMocks = vi.hoisted(() => ({
  listVideos: vi.fn(),
  login: vi.fn(),
  createVideo: vi.fn(),
  uploadBytes: vi.fn(),
  completeUpload: vi.fn(),
}))

vi.mock('./api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('./api')>()
  return { ...actual, api: apiMocks }
})

describe('OpenTube feed and creator flows', () => {
  beforeEach(() => {
    sessionStorage.clear()
    vi.stubGlobal('IntersectionObserver', class {
      observe() {}
      unobserve() {}
      disconnect() {}
      takeRecords() { return [] }
    })
    apiMocks.listVideos.mockReset().mockResolvedValue({ items: [first, second], next_cursor: 'page-2' })
    apiMocks.login.mockReset().mockResolvedValue({ access_token: 'demo-token', expires_in: 3600 })
    apiMocks.createVideo.mockReset().mockResolvedValue({ id: 'new-video', upload_url: 'https://media.test/upload' })
    apiMocks.uploadBytes.mockReset().mockResolvedValue(undefined)
    apiMocks.completeUpload.mockReset().mockResolvedValue({ id: 'new-video', status: 'uploaded' })
  })

  it('renders a video page and appends the next page without duplicates', async () => {
    const wrapper = mount(App)
    await flushPromises()
    expect(wrapper.findAll('.video-card')).toHaveLength(2)

    apiMocks.listVideos.mockResolvedValueOnce({ items: [second, third], next_cursor: null })
    await wrapper.get('.load-more').trigger('click')
    await flushPromises()

    expect(wrapper.findAll('.video-card')).toHaveLength(3)
    expect(wrapper.findAll('.card-title').map((button) => button.text())).toEqual(['Quiet Lake', 'Small Observatory', 'Night Sky'])
    wrapper.unmount()
  })

  it('submits search text and labels the result set', async () => {
    const wrapper = mount(App)
    await flushPromises()
    apiMocks.listVideos.mockResolvedValueOnce({ items: [third], next_cursor: null })

    await wrapper.get('input#video-search').setValue('night sky')
    await wrapper.get('form[role="search"]').trigger('submit')
    await flushPromises()

    expect(wrapper.get('#feed-heading').text()).toContain('night sky')
    expect(apiMocks.listVideos).toHaveBeenLastCalledWith('night sky', null)
    wrapper.unmount()
  })

  it('shows an empty feed and opens the creator sign-in dialog', async () => {
    apiMocks.listVideos.mockResolvedValueOnce({ items: [], next_cursor: null })
    const wrapper = mount(App)
    await flushPromises()

    expect(wrapper.get('.empty-state').text()).toContain('No videos here yet')
    await wrapper.get('.empty-state .text-action').trigger('click')
    expect(wrapper.get('dialog').text()).toContain('This local MVP has no signup')
    expect((wrapper.get('dialog input[type="email"]').element as HTMLInputElement).value).toBe('creator@opentube.local')
    wrapper.unmount()
  })

  it('shows a feed error and retries from the first page', async () => {
    apiMocks.listVideos.mockRejectedValueOnce(new Error('feed unavailable'))
    const wrapper = mount(App)
    await flushPromises()
    expect(wrapper.get('.notice').text()).toContain('feed unavailable')

    apiMocks.listVideos.mockResolvedValueOnce({ items: [third], next_cursor: null })
    await wrapper.get('.notice button').trigger('click')
    await flushPromises()

    expect(wrapper.findAll('.video-card')).toHaveLength(1)
    wrapper.unmount()
  })

  it('shows the persistent signed-in indicator after a successful login', async () => {
    const wrapper = mount(App)
    await flushPromises()
    await wrapper.get('button[aria-label^="Sign in"]').trigger('click')
    await wrapper.get('input[type="password"]').setValue('synthetic-test-password')
    await wrapper.get('dialog form').trigger('submit')
    await flushPromises()

    expect(wrapper.text()).toContain('Signed in')
    expect(wrapper.find('dialog').exists()).toBe(false)
    expect(sessionStorage.getItem('opentube-token')).toBe('demo-token')
    wrapper.unmount()
  })

  it('keeps sign-in open and explains invalid creator credentials', async () => {
    apiMocks.login.mockRejectedValueOnce(new Error('Email or password did not match'))
    const wrapper = mount(App)
    await flushPromises()
    await wrapper.get('button[aria-label^="Sign in"]').trigger('click')
    await wrapper.get('input[type="password"]').setValue('wrong')
    await wrapper.get('dialog form').trigger('submit')
    await flushPromises()

    expect(wrapper.get('dialog [role="alert"]').text()).toContain('Email or password did not match')
    expect(wrapper.find('dialog').exists()).toBe(true)
    wrapper.unmount()
  })

  it('opens sign-in when a visitor selects upload', async () => {
    const wrapper = mount(App)
    await flushPromises()
    await wrapper.get('.upload-button').trigger('click')
    expect(wrapper.get('dialog h2').text()).toBe('Welcome back.')
    wrapper.unmount()
  })

  it('rejects a non-MP4 selection before creating an upload', async () => {
    const wrapper = await openAuthenticatedUpload()
    await selectFile(wrapper, new File(['not a video'], 'note.txt', { type: 'text/plain' }))
    await wrapper.get('.upload-dialog form').trigger('submit')

    expect(wrapper.get('.upload-status').text()).toContain('Choose a non-empty MP4')
    expect(apiMocks.createVideo).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('uploads an MP4 through storage and submits it for processing', async () => {
    const wrapper = await openAuthenticatedUpload()
    await wrapper.get('.upload-dialog input[placeholder="Give your video a title"]').setValue('A test video')
    await selectFile(wrapper, new File(['synthetic video'], 'sample.mp4', { type: 'video/mp4' }))
    await wrapper.get('.upload-dialog form').trigger('submit')
    await flushPromises()

    expect(apiMocks.createVideo).toHaveBeenCalledWith({ title: 'A test video', description: '', content_type: 'video/mp4', size_bytes: 15 }, 'demo-token')
    expect(apiMocks.uploadBytes).toHaveBeenCalledWith('https://media.test/upload', expect.any(File))
    expect(apiMocks.completeUpload).toHaveBeenCalledWith('new-video', 'demo-token')
    expect(wrapper.get('.upload-status').text()).toContain('Upload received')
    wrapper.unmount()
  })

  it('surfaces media-storage errors in the upload dialog', async () => {
    apiMocks.uploadBytes.mockRejectedValueOnce(new Error('Could not reach media storage'))
    const wrapper = await openAuthenticatedUpload()
    await wrapper.get('.upload-dialog input[placeholder="Give your video a title"]').setValue('A test video')
    await selectFile(wrapper, new File(['synthetic video'], 'sample.mp4', { type: 'video/mp4' }))
    await wrapper.get('.upload-dialog form').trigger('submit')
    await flushPromises()

    expect(wrapper.get('.upload-status').text()).toContain('Could not reach media storage')
    expect(apiMocks.completeUpload).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('opens and closes the HLS playback view', async () => {
    const wrapper = mount(App)
    await flushPromises()
    await wrapper.get('button[aria-label="Play Quiet Lake"]').trigger('click')
    await flushPromises()

    expect(wrapper.find('.watch-page').exists()).toBe(true)
    expect(wrapper.get('.watch-page h1').text()).toBe('Quiet Lake')
    await wrapper.get('.back-link').trigger('click')
    expect(wrapper.find('.watch-page').exists()).toBe(false)
    wrapper.unmount()
  })
})

async function openAuthenticatedUpload(): Promise<VueWrapper> {
  const wrapper = mount(App)
  await flushPromises()
  await wrapper.get('button[aria-label^="Sign in"]').trigger('click')
  await wrapper.get('input[type="password"]').setValue('synthetic-test-password')
  await wrapper.get('dialog form').trigger('submit')
  await flushPromises()
  await wrapper.get('.upload-button').trigger('click')
  return wrapper
}

async function selectFile(wrapper: VueWrapper, file: File) {
  const input = wrapper.get('.upload-dialog input[type="file"]')
  Object.defineProperty(input.element, 'files', { configurable: true, value: [file] })
  await input.trigger('change')
}
