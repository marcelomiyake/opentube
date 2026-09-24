<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import Hls from 'hls.js'
import { api, appendUniqueVideos, type VideoSummary } from './api'
import { registerPublicVideoSearchTool, type ToolRegistration } from './webmcp'

const videos = ref<VideoSummary[]>([])
const cursor = ref<string | null>(null)
const query = ref('')
const searchInput = ref('')
const activeCategory = ref('For you')
const loading = ref(false)
const exhausted = ref(false)
const message = ref('')
const token = ref(sessionStorage.getItem('opentube-token') ?? '')
const showLogin = ref(false)
const showUpload = ref(false)
const signingIn = ref(false)
const uploadInProgress = ref(false)
const email = ref('creator@opentube.local')
const password = ref('')
const loginError = ref('')
const uploadTitle = ref('')
const uploadDescription = ref('')
const uploadFile = ref<File | null>(null)
const uploadStatus = ref('')
const activeVideo = ref<VideoSummary | null>(null)
const player = ref<HTMLVideoElement | null>(null)
const sentinel = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | undefined
let hls: Hls | undefined
let searchTimer: ReturnType<typeof setTimeout> | undefined
let webmcpRegistration: ToolRegistration | null = null

const categories = ['For you', 'Design', 'Outdoors', 'Music', 'Science', 'Food', 'Short films']
const uploadSizeLabel = computed(() => uploadFile.value ? `${(uploadFile.value.size / 1024 / 1024).toFixed(1)} MB` : 'MP4 up to 1 GB')
const canUpload = computed(() => Boolean(token.value))

async function loadPage(reset = false) {
  if (loading.value || (!reset && exhausted.value)) return
  loading.value = true
  message.value = ''
  if (reset) {
    videos.value = []
    cursor.value = null
    exhausted.value = false
  }
  try {
    const page = await api.listVideos(query.value, cursor.value)
    videos.value = reset ? appendUniqueVideos([], page.items) : appendUniqueVideos(videos.value, page.items)
    cursor.value = page.next_cursor
    exhausted.value = !page.next_cursor || page.items.length === 0
  } catch (error) {
    message.value = error instanceof Error ? error.message : 'The feed could not be loaded.'
  } finally {
    loading.value = false
  }
}

function search() {
  query.value = searchInput.value.trim()
  void loadPage(true)
}

function selectCategory(category: string) {
  activeCategory.value = category
  if (category === 'For you') {
    searchInput.value = ''
    query.value = ''
    void loadPage(true)
    return
  }
  searchInput.value = category
  search()
}

async function signIn() {
  loginError.value = ''
  signingIn.value = true
  try {
    const result = await api.login(email.value, password.value)
    token.value = result.access_token
    sessionStorage.setItem('opentube-token', result.access_token)
    password.value = ''
    showLogin.value = false
  } catch (error) {
    const message = error instanceof Error ? error.message : ''
    loginError.value = message.startsWith('Email or password')
      ? message
      : 'OpenTube could not complete sign-in. Check that local services are running and try again.'
  } finally {
    signingIn.value = false
  }
}

function signOut() {
  token.value = ''
  sessionStorage.removeItem('opentube-token')
}

function chooseFile(event: Event) {
  const input = event.target as HTMLInputElement
  uploadFile.value = input.files?.[0] ?? null
}

function openUpload() {
  uploadStatus.value = ''
  showUpload.value = true
}

async function submitUpload() {
  if (!token.value) {
    showUpload.value = false
    showLogin.value = true
    return
  }
  const file = uploadFile.value
  if (!file || file.type !== 'video/mp4' || file.size === 0 || file.size > 1_073_741_824) {
    uploadStatus.value = 'Choose a non-empty MP4 file no larger than 1 GB.'
    return
  }
  if (!uploadTitle.value.trim()) {
    uploadStatus.value = 'Add a title before uploading.'
    return
  }
  uploadStatus.value = 'Creating upload…'
  uploadInProgress.value = true
  try {
    const created = await api.createVideo({ title: uploadTitle.value.trim(), description: uploadDescription.value.trim(), content_type: 'video/mp4', size_bytes: file.size }, token.value)
    uploadStatus.value = 'Uploading video…'
    await api.uploadBytes(created.upload_url, file)
    uploadStatus.value = 'Submitting for processing…'
    await api.completeUpload(created.id, token.value)
    uploadStatus.value = 'Upload received. It will appear after processing.'
    uploadTitle.value = ''
    uploadDescription.value = ''
    uploadFile.value = null
    await loadPage(true)
  } catch (error) {
    uploadStatus.value = error instanceof Error ? error.message : 'Upload failed. Try again.'
  } finally {
    uploadInProgress.value = false
  }
}

async function watchVideo(video: VideoSummary) {
  activeVideo.value = video
  await nextTick()
  if (!player.value) return
  hls?.destroy()
  hls = undefined
  if (player.value.canPlayType('application/vnd.apple.mpegurl')) {
    player.value.src = video.playback_url
  } else if (Hls.isSupported()) {
    hls = new Hls({ enableWorker: true })
    hls.loadSource(video.playback_url)
    hls.attachMedia(player.value)
  } else {
    message.value = 'This browser cannot play HLS video.'
  }
}

function closeVideo() {
  hls?.destroy()
  hls = undefined
  activeVideo.value = null
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat(undefined, { month: 'short', day: 'numeric' }).format(new Date(value))
}

onMounted(() => {
  webmcpRegistration = registerPublicVideoSearchTool(document.modelContext, api.listVideos)
  void loadPage(true)
  observer = new IntersectionObserver((entries) => {
    if (entries.some((entry) => entry.isIntersecting)) void loadPage()
  }, { rootMargin: '700px 0px' })
  if (sentinel.value) observer.observe(sentinel.value)
})

watch(sentinel, (element) => {
  if (element && observer) observer.observe(element)
})

watch(searchInput, () => {
  clearTimeout(searchTimer)
  searchTimer = setTimeout(search, 350)
})

onBeforeUnmount(() => {
  webmcpRegistration?.unregister()
  observer?.disconnect()
  hls?.destroy()
  clearTimeout(searchTimer)
})
</script>

<template>
  <div class="app-shell">
    <header class="topbar">
      <a class="brand" href="#top" aria-label="OpenTube home" @click.prevent="closeVideo">
        <span class="brand-mark" aria-hidden="true"><span></span></span>
        <span>open<span class="brand-light">tube</span></span>
      </a>
      <form class="searchbar" role="search" @submit.prevent="search">
        <label class="visually-hidden" for="video-search">Search videos</label>
        <input id="video-search" v-model="searchInput" placeholder="Search anything worth watching" autocomplete="off" />
        <button class="search-button" type="submit" aria-label="Search videos"><span aria-hidden="true">⌕</span></button>
      </form>
      <div class="top-actions">
        <output v-if="token" class="auth-state">Signed in</output>
        <button class="upload-button" @click="canUpload ? openUpload() : (showLogin = true)"><span aria-hidden="true">＋</span> Upload</button>
        <button v-if="!token" class="avatar" aria-label="Sign in (MM)" @click="showLogin = true">MM</button>
        <button v-else class="avatar signed-in" aria-label="Sign out (MM)" title="Signed in as demo creator · Sign out" @click="signOut">MM</button>
      </div>
    </header>

    <div class="page-layout" id="top">
      <aside class="sidebar" aria-label="Video categories">
        <div class="sidebar-heading">EXPLORE</div>
        <button v-for="(category, index) in categories" :key="category" class="nav-item" :class="{ active: activeCategory === category }" @click="selectCategory(category)">
          <span class="nav-icon" aria-hidden="true">{{ ['◉','✳','⌁','♫','⌬','◒','▣'][index] }}</span>{{ category }}
        </button>
        <div class="sidebar-rule"></div>
        <div class="sidebar-note"><span class="live-dot"></span><span>Made for curious minds</span></div>
        <p class="sidebar-foot">An independent learning project.<br />Built for the joy of finding something new.</p>
      </aside>

      <main class="main-content">
        <section v-if="!activeVideo" class="welcome-strip" aria-label="OpenTube introduction">
          <div class="welcome-copy">
            <p class="eyebrow"><span class="eyebrow-line"></span> A LITTLE MORE WONDER</p>
            <h1>Stories take you<br /><em>somewhere.</em></h1>
            <p class="welcome-body">Find a new perspective, follow a curious idea, or just stay for the view.</p>
          </div>
          <div class="orbit-art" aria-hidden="true">
            <div class="orbit orbit-one"></div><div class="orbit orbit-two"></div>
            <div class="planet planet-large"></div><div class="planet planet-small"></div>
            <span class="star star-one">✳</span><span class="star star-two">✦</span>
          </div>
          <span class="welcome-caption">OPEN TUBE / 001</span>
        </section>

        <section v-if="activeVideo" class="watch-page">
          <button class="back-link" @click="closeVideo">← Back to videos</button>
          <div class="player-frame"><video ref="player" controls playsinline autoplay aria-label="Video player"></video></div>
          <div class="watch-meta"><div><p class="eyebrow">NOW PLAYING</p><h1>{{ activeVideo.title }}</h1><p>{{ activeVideo.description }}</p></div><div class="creator-chip"><span class="creator-avatar">{{ activeVideo.creator.slice(0, 1).toUpperCase() }}</span><span>{{ activeVideo.creator }}</span></div></div>
        </section>

        <section v-else class="feed-section" aria-labelledby="feed-heading">
          <div class="section-heading">
            <div><p class="eyebrow">THE OPEN FEED</p><h2 id="feed-heading">{{ query ? `Results for “${query}”` : 'Picked for your next pause' }}</h2></div>
            <span v-if="videos.length" class="result-count">{{ videos.length }} videos loaded</span>
          </div>
          <div v-if="message" class="notice" role="alert"><span>{{ message }}</span><button @click="loadPage(true)">Try again</button></div>
          <output v-else-if="loading && videos.length === 0" class="loading-state"><span class="spinner"></span> Finding something good…</output>
          <div v-else-if="!loading && !videos.length" class="empty-state"><span class="empty-art" aria-hidden="true">✳</span><h3>No videos here yet</h3><p>Try another search, or be the first creator to add a story.</p><button class="text-action" @click="showLogin = true">Sign in to upload <span aria-hidden="true">→</span></button></div>
          <div v-else class="video-grid">
            <article v-for="(video, index) in videos" :key="video.id" class="video-card">
              <button class="thumbnail" :class="`tone-${index % 6}`" :aria-label="`Play ${video.title}`" @click="watchVideo(video)">
                <span class="thumb-orbit" aria-hidden="true"></span><span class="thumb-title">{{ video.title.slice(0, 1) }}</span>
                <span class="play-control" aria-hidden="true">▶</span><span class="duration">OPEN TUBE</span>
              </button>
              <div class="card-meta"><span class="card-avatar" aria-hidden="true">{{ video.creator.slice(0, 1).toUpperCase() }}</span><div class="card-copy"><button class="card-title" @click="watchVideo(video)">{{ video.title }}</button><p>{{ video.creator }}</p><p>{{ formatDate(video.published_at) }} <span class="meta-dot">·</span> OpenTube original</p></div><button class="more-button" :aria-label="`More options for ${video.title}`">···</button></div>
            </article>
          </div>
          <div ref="sentinel" class="feed-sentinel" aria-hidden="true"></div>
          <div class="feed-footer">
            <output v-if="loading && videos.length" class="loading-inline"><span class="spinner small"></span> Loading more</output>
            <button v-if="!exhausted && !loading" class="load-more" @click="loadPage()">Load more videos <span aria-hidden="true">↓</span></button>
            <p v-if="exhausted && videos.length" class="end-note">You’re all caught up <span>✦</span></p>
          </div>
        </section>
        <footer class="site-footer"><span>OPENTUBE</span><span>Small moments. Big worlds.</span><span>LOCAL LEARNING PROJECT</span></footer>
      </main>
    </div>

    <div v-if="showLogin" class="modal-backdrop" @click.self="showLogin = false">
      <dialog open class="dialog" aria-modal="true" aria-labelledby="login-title">
        <button class="dialog-close" aria-label="Close sign in" @click="showLogin = false">×</button><p class="eyebrow">CREATOR ACCESS</p><h2 id="login-title">Welcome back.</h2><p class="dialog-help">This local MVP has no signup. Use the creator email and password stored in your ignored <code>.env</code> file.</p>
        <form class="dialog-form" @submit.prevent="signIn"><label>Email<input v-model="email" type="email" autocomplete="username" required /></label><label>Password<input v-model="password" type="password" autocomplete="current-password" required /></label><p v-if="loginError" class="form-error" role="alert">{{ loginError }}</p><button class="primary-button" type="submit" :disabled="signingIn">{{ signingIn ? 'Signing in…' : 'Sign in' }} <span aria-hidden="true">→</span></button></form>
      </dialog>
    </div>

    <div v-if="showUpload" class="modal-backdrop" @click.self="showUpload = false">
      <dialog open class="dialog upload-dialog" aria-modal="true" aria-labelledby="upload-title">
        <button class="dialog-close" aria-label="Close upload" @click="showUpload = false">×</button><p class="eyebrow">SHARE A STORY</p><h2 id="upload-title">Upload a video.</h2><p class="dialog-help">MP4 only · up to 1 GB · public after processing</p>
        <form class="dialog-form" @submit.prevent="submitUpload"><label>Title<input v-model="uploadTitle" maxlength="120" required placeholder="Give your video a title" /></label><label>Description <span class="optional">OPTIONAL</span><textarea v-model="uploadDescription" maxlength="2000" rows="3" placeholder="What should people know?"></textarea></label><label class="file-picker">{{ uploadFile ? uploadFile.name : 'Choose an MP4 file' }}<span>{{ uploadSizeLabel }}</span><input type="file" accept="video/mp4,.mp4" @change="chooseFile" :disabled="uploadInProgress" /></label><output v-if="uploadStatus" class="upload-status">{{ uploadStatus }}</output><button class="primary-button" :disabled="uploadInProgress" :type="uploadStatus.startsWith('Upload received') ? 'button' : 'submit'" @click="uploadStatus.startsWith('Upload received') && (showUpload = false)">{{ uploadStatus.startsWith('Upload received') ? 'Done' : uploadInProgress ? 'Uploading…' : 'Upload video' }} <span aria-hidden="true">→</span></button></form>
      </dialog>
    </div>
  </div>
</template>
