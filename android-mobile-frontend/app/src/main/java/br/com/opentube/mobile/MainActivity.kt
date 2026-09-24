package br.com.opentube.mobile

import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.common.MediaItem
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.PlayerView
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

private data class FeedLoadResult(
    val videos: List<VideoSummary>,
    val cursor: String?,
    val exhausted: Boolean,
    val status: String,
)

private suspend fun fetchFeedPage(
    search: String,
    cursor: String?,
    currentVideos: List<VideoSummary>,
): FeedLoadResult = try {
    val page = withContext(Dispatchers.IO) { apiClient.listVideos(search, cursor) }
    val updatedVideos = appendUniqueVideos(currentVideos, page.items)
    FeedLoadResult(
        videos = updatedVideos,
        cursor = page.nextCursor,
        exhausted = page.nextCursor == null || page.items.isEmpty(),
        status = if (updatedVideos.isEmpty()) "No videos yet. Upload one from the phone or web client." else "",
    )
} catch (error: CancellationException) {
    throw error
} catch (error: Exception) {
    FeedLoadResult(
        videos = currentVideos,
        cursor = cursor,
        exhausted = false,
        status = "Could not load videos. Pull down and try again.",
    )
}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme(colorScheme = darkColorScheme(primary = Color(0xFFFF5146), background = Color(0xFF101010), surface = Color(0xFF1B1B1B))) {
                MobileApp()
            }
        }
    }
}

@Composable
private fun MobileApp() {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val preferences = remember { context.getSharedPreferences("opentube", 0) }
    val listState = rememberLazyListState()
    var token by remember { mutableStateOf(preferences.getString("token", "").orEmpty()) }
    var email by remember { mutableStateOf("creator@opentube.local") }
    var password by remember { mutableStateOf("") }
    var title by remember { mutableStateOf("") }
    var description by remember { mutableStateOf("") }
    var selectedUri by remember { mutableStateOf<Uri?>(null) }
    var videos by remember { mutableStateOf(emptyList<VideoSummary>()) }
    var cursor by remember { mutableStateOf<String?>(null) }
    var search by remember { mutableStateOf("") }
    var activeVideo by remember { mutableStateOf<VideoSummary?>(null) }
    var status by remember { mutableStateOf("Loading videos…") }
    var loading by remember { mutableStateOf(false) }
    var exhausted by remember { mutableStateOf(false) }

    suspend fun loadPage(reset: Boolean) {
        if (loading || (!reset && exhausted)) return
        loading = true
        if (reset) {
            videos = emptyList()
            cursor = null
            exhausted = false
        }
        try {
            val result = fetchFeedPage(search, cursor, videos)
            videos = result.videos
            cursor = result.cursor
            exhausted = result.exhausted
            status = result.status
        } finally {
            loading = false
        }
    }

    LaunchedEffect(Unit) { loadPage(reset = true) }
    LaunchedEffect(listState.layoutInfo.visibleItemsInfo.lastOrNull()?.index, videos.size, cursor, loading) {
        val lastVisible = listState.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0
        if (videos.isNotEmpty() && lastVisible >= videos.size - 3 && !loading && !exhausted) loadPage(reset = false)
    }

    val filePicker = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri -> selectedUri = uri }
    val player = remember(activeVideo?.playbackUrl) {
        activeVideo?.let { video ->
            ExoPlayer.Builder(context).build().apply {
                setMediaItem(MediaItem.fromUri(video.playbackUrl))
                prepare()
                playWhenReady = true
            }
        }
    }
    DisposableEffect(player) { onDispose { player?.release() } }

    fun signIn() {
        status = "Signing in…"
        scope.launch {
            runCatching { withContext(Dispatchers.IO) { apiClient.login(email.trim(), password) } }
                .onSuccess {
                    token = it
                    preferences.edit().putString("token", it).apply()
                    password = ""
                    status = "Signed in as creator."
                }
                .onFailure { status = "Sign-in failed. Check the local demo credentials." }
        }
    }

    fun uploadVideo() {
        val uri = selectedUri
        if (uri == null || title.isBlank()) {
            status = "Add a title and choose an MP4 first."
            return
        }
        status = "Uploading…"
        scope.launch {
            runCatching {
                uploadSelectedVideo(context.contentResolver, uri, token, title.trim(), description.trim())
            }.onSuccess {
                title = ""
                description = ""
                selectedUri = null
                status = "Upload received. It will appear after processing."
            }.onFailure { status = it.message ?: "Upload failed. Please try again." }
        }
    }

    Scaffold(containerColor = Color(0xFF101010)) { padding ->
        Column(Modifier.fillMaxSize().padding(padding).padding(horizontal = 18.dp)) {
            MobileHeader(
                hasToken = token.isNotBlank(),
                onSignIn = { status = "Enter the seeded creator credentials below." },
                onSignOut = { token = ""; preferences.edit().remove("token").apply() },
            )
            VideoSearchBar(search, onSearchChange = { search = it }, onSearch = { scope.launch { loadPage(reset = true) } })
            Spacer(Modifier.height(10.dp))
            FeedHeading(onRefresh = { scope.launch { loadPage(reset = true) } })
            CreatorPanel(
                state = CreatorPanelState(
                    hasToken = token.isNotBlank(),
                    email = email,
                    password = password,
                    title = title,
                    description = description,
                    hasSelectedUri = selectedUri != null,
                ),
                actions = CreatorPanelActions(
                    onEmailChange = { email = it },
                    onPasswordChange = { password = it },
                    onTitleChange = { title = it },
                    onDescriptionChange = { description = it },
                    onChooseFile = { filePicker.launch(arrayOf("video/mp4")) },
                    onSignIn = ::signIn,
                    onUpload = ::uploadVideo,
                ),
            )
            StatusMessage(status)
            MobileVideoFeed(videos, listState, loading) { activeVideo = it }
        }
    }

    VideoPlaybackDialog(activeVideo, player) { activeVideo = null }
}

private suspend fun uploadSelectedVideo(
    resolver: android.content.ContentResolver,
    uri: Uri,
    token: String,
    title: String,
    description: String,
) = withContext(Dispatchers.IO) {
    val size = resolver.openFileDescriptor(uri, "r")?.use { it.statSize } ?: -1
    require(size in 1..1_073_741_824) { "Choose an MP4 no larger than 1 GB." }
    val (id, uploadUrl) = apiClient.createVideo(token, title, description, size)
    apiClient.upload(resolver, uri, uploadUrl, size)
    apiClient.completeUpload(token, id)
}

@Composable
private fun MobileHeader(hasToken: Boolean, onSignIn: () -> Unit, onSignOut: () -> Unit) {
    Row(Modifier.fillMaxWidth().padding(top = 14.dp, bottom = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {
        Text("OpenTube", style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Black, color = Color.White)
        if (hasToken) TextButton(onClick = onSignOut) { Text("Sign out") }
        else TextButton(onClick = onSignIn) { Text("Sign in") }
    }
}

@Composable
private fun VideoSearchBar(search: String, onSearchChange: (String) -> Unit, onSearch: () -> Unit) {
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        OutlinedTextField(value = search, onValueChange = onSearchChange, modifier = Modifier.weight(1f), singleLine = true, label = { Text("Search videos") })
        Button(onClick = onSearch) { Text("Search") }
    }
}

@Composable
private fun FeedHeading(onRefresh: () -> Unit) {
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        Text("Public feed", style = MaterialTheme.typography.titleLarge, color = Color.White, fontWeight = FontWeight.Bold)
        Spacer(Modifier.weight(1f))
        TextButton(onClick = onRefresh) { Text("Refresh") }
    }
}

private data class CreatorPanelState(
    val hasToken: Boolean,
    val email: String,
    val password: String,
    val title: String,
    val description: String,
    val hasSelectedUri: Boolean,
)

private data class CreatorPanelActions(
    val onEmailChange: (String) -> Unit,
    val onPasswordChange: (String) -> Unit,
    val onTitleChange: (String) -> Unit,
    val onDescriptionChange: (String) -> Unit,
    val onChooseFile: () -> Unit,
    val onSignIn: () -> Unit,
    val onUpload: () -> Unit,
)

@Composable
private fun CreatorPanel(state: CreatorPanelState, actions: CreatorPanelActions) {
    if (state.hasToken) {
        UploadForm(
            state.title,
            state.description,
            state.hasSelectedUri,
            actions.onTitleChange,
            actions.onDescriptionChange,
            actions.onChooseFile,
            actions.onUpload,
        )
    } else {
        SignInForm(state.email, state.password, actions.onEmailChange, actions.onPasswordChange, actions.onSignIn)
    }
}

@Composable
private fun SignInForm(
    email: String,
    password: String,
    onEmailChange: (String) -> Unit,
    onPasswordChange: (String) -> Unit,
    onSignIn: () -> Unit,
) {
    Text("Creator sign-in", color = Color(0xFFBDBDBD), style = MaterialTheme.typography.titleSmall)
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        OutlinedTextField(email, onEmailChange, modifier = Modifier.weight(1f), singleLine = true, label = { Text("Email") })
        OutlinedTextField(password, onPasswordChange, modifier = Modifier.weight(1f), singleLine = true, label = { Text("Password") }, visualTransformation = PasswordVisualTransformation(), keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password))
    }
    Button(onClick = onSignIn) { Text("Sign in") }
}

@Composable
private fun UploadForm(
    title: String,
    description: String,
    hasSelectedUri: Boolean,
    onTitleChange: (String) -> Unit,
    onDescriptionChange: (String) -> Unit,
    onChooseFile: () -> Unit,
    onUpload: () -> Unit,
) {
    OutlinedTextField(title, onTitleChange, modifier = Modifier.fillMaxWidth(), singleLine = true, label = { Text("Video title") })
    OutlinedTextField(description, onDescriptionChange, modifier = Modifier.fillMaxWidth(), label = { Text("Description") })
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        Button(onClick = onChooseFile) { Text(if (hasSelectedUri) "Choose another" else "Choose MP4") }
        TextButton(onClick = onUpload) { Text("Upload") }
        if (hasSelectedUri) Text("MP4 selected", color = Color(0xFFBDBDBD))
    }
}

@Composable
private fun StatusMessage(status: String) {
    if (status.isNotBlank()) Text(status, Modifier.padding(vertical = 6.dp), color = Color(0xFFBDBDBD), style = MaterialTheme.typography.bodySmall)
}

@Composable
private fun MobileVideoFeed(
    videos: List<VideoSummary>,
    listState: androidx.compose.foundation.lazy.LazyListState,
    loading: Boolean,
    onVideoSelected: (VideoSummary) -> Unit,
) {
    LazyColumn(state = listState, modifier = Modifier.fillMaxSize(), verticalArrangement = Arrangement.spacedBy(12.dp), contentPadding = androidx.compose.foundation.layout.PaddingValues(vertical = 12.dp)) {
        items(videos, key = { it.id }) { video -> VideoCard(video, onClick = { onVideoSelected(video) }) }
        if (loading) item { Text("Loading more…", Modifier.padding(12.dp), color = Color.LightGray) }
    }
}

@Composable
private fun VideoPlaybackDialog(video: VideoSummary?, player: ExoPlayer?, onClose: () -> Unit) {
    if (video == null || player == null) return
    AlertDialog(
        onDismissRequest = onClose,
        confirmButton = { TextButton(onClick = onClose) { Text("Close") } },
        title = { Text(video.title) },
        text = { AndroidView(factory = { PlayerView(it).apply { this.player = player; useController = true } }, modifier = Modifier.fillMaxWidth().height(220.dp)) },
    )
}

@Composable
private fun VideoCard(video: VideoSummary, onClick: () -> Unit) {
    Card(onClick = onClick, modifier = Modifier.fillMaxWidth(), shape = RoundedCornerShape(18.dp), colors = CardDefaults.cardColors(containerColor = Color(0xFF202020))) {
        Column {
            Box(Modifier.fillMaxWidth().height(160.dp).background(Color(0xFF292522)), contentAlignment = Alignment.Center) {
                Text("▶", color = Color(0xFFFF5146), style = MaterialTheme.typography.displaySmall)
            }
            Column(Modifier.padding(14.dp)) {
                Text(video.title, color = Color.White, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                Text(video.creator, color = Color(0xFFB6B6B6), style = MaterialTheme.typography.bodySmall)
                if (video.description.isNotBlank()) Text(video.description, color = Color(0xFFD0D0D0), style = MaterialTheme.typography.bodyMedium, maxLines = 2)
            }
        }
    }
}
