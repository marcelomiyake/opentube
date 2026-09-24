package br.com.opentube.tv

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.common.MediaItem
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.PlayerView
import androidx.tv.material3.MaterialTheme as TvMaterialTheme
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
        status = if (updatedVideos.isEmpty()) "No videos found. Search or refresh the feed." else "",
    )
} catch (error: CancellationException) {
    throw error
} catch (error: Exception) {
    FeedLoadResult(
        videos = currentVideos,
        cursor = cursor,
        exhausted = false,
        status = "Could not load videos. Try again.",
    )
}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            TvMaterialTheme {
                MaterialTheme(colorScheme = androidx.compose.material3.darkColorScheme(primary = Color(0xFFFF5146), background = Color(0xFF101010), surface = Color(0xFF202020))) {
                    TvApp()
                }
            }
        }
    }
}

@Composable
private fun TvApp() {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val rowState = rememberLazyListState()
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
        if (reset) { videos = emptyList(); cursor = null; exhausted = false }
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
    LaunchedEffect(rowState.layoutInfo.visibleItemsInfo.lastOrNull()?.index, videos.size, cursor, loading) {
        val last = rowState.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0
        if (videos.isNotEmpty() && last >= videos.size - 4 && !loading && !exhausted) loadPage(reset = false)
    }

    val player = remember(activeVideo?.playbackUrl) {
        activeVideo?.let { video -> ExoPlayer.Builder(context).build().apply { setMediaItem(MediaItem.fromUri(video.playbackUrl)); prepare(); playWhenReady = true } }
    }
    DisposableEffect(player) { onDispose { player?.release() } }

    Column(Modifier.fillMaxSize().background(Color(0xFF101010)).padding(horizontal = 52.dp, vertical = 28.dp)) {
        TvHeader(onRefresh = { scope.launch { loadPage(reset = true) } })
        Spacer(Modifier.height(22.dp))
        TvSearchBox(search, onSearchChange = { search = it }, onSearch = { scope.launch { loadPage(reset = true) } })
        Spacer(Modifier.height(20.dp))
        Text(if (search.isBlank()) "For you" else "Search results", color = Color.White, fontSize = 24.sp, fontWeight = FontWeight.Bold)
        Text(status, color = Color(0xFFB9B9B9), fontSize = 15.sp, modifier = Modifier.padding(top = 6.dp))
        Spacer(Modifier.height(10.dp))
        TvVideoFeed(videos, rowState, loading) { activeVideo = it }
    }

    TvPlaybackDialog(activeVideo, player) { activeVideo = null }
}

@Composable
private fun TvHeader(onRefresh: () -> Unit) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
        Column {
            Text("OpenTube", color = Color.White, fontSize = 34.sp, fontWeight = FontWeight.Black)
            Text("Discover something worth watching", color = Color(0xFFB9B9B9), fontSize = 17.sp)
        }
        Button(onClick = onRefresh) { Text("Refresh") }
    }
}

@Composable
private fun TvSearchBox(search: String, onSearchChange: (String) -> Unit, onSearch: () -> Unit) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        BasicTextField(
            value = search,
            onValueChange = onSearchChange,
            modifier = Modifier.width(440.dp).background(Color(0xFF252525), RoundedCornerShape(12.dp)).padding(horizontal = 18.dp, vertical = 16.dp).focusable(),
            singleLine = true,
            textStyle = androidx.compose.ui.text.TextStyle(color = Color.White, fontSize = 19.sp),
            decorationBox = { inner ->
                if (search.isEmpty()) Text("Search videos", color = Color(0xFF888888), fontSize = 19.sp)
                inner()
            },
        )
        Spacer(Modifier.width(12.dp))
        Button(onClick = onSearch) { Text("Search") }
    }
}

@Composable
private fun TvVideoFeed(
    videos: List<VideoSummary>,
    rowState: androidx.compose.foundation.lazy.LazyListState,
    loading: Boolean,
    onVideoSelected: (VideoSummary) -> Unit,
) {
    LazyRow(state = rowState, modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(18.dp)) {
        items(videos, key = { it.id }) { video ->
            TvVideoCard(video) { onVideoSelected(video) }
        }
        if (loading) item { Text("Loading…", color = Color.LightGray, modifier = Modifier.padding(30.dp)) }
    }
}

@Composable
private fun TvVideoCard(video: VideoSummary, onClick: () -> Unit) {
    Card(
        onClick = onClick,
        modifier = Modifier.width(300.dp).height(252.dp),
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(containerColor = Color(0xFF202020)),
    ) {
        Column {
            Box(Modifier.fillMaxWidth().height(164.dp).background(Color(0xFF2A2422)), contentAlignment = Alignment.Center) {
                Text("▶", color = Color(0xFFFF5146), fontSize = 42.sp)
            }
            Column(Modifier.padding(14.dp)) {
                Text(video.title, color = Color.White, fontSize = 18.sp, fontWeight = FontWeight.Bold, maxLines = 1)
                Text(video.creator, color = Color(0xFFB9B9B9), fontSize = 14.sp, maxLines = 1)
            }
        }
    }
}

@Composable
private fun TvPlaybackDialog(video: VideoSummary?, player: ExoPlayer?, onClose: () -> Unit) {
    if (video == null || player == null) return
    androidx.compose.ui.window.Dialog(onDismissRequest = onClose, properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)) {
        Column(Modifier.fillMaxSize().background(Color.Black).padding(48.dp), verticalArrangement = Arrangement.Center) {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                Text(video.title, color = Color.White, fontSize = 26.sp, fontWeight = FontWeight.Bold)
                Button(onClick = onClose) { Text("Back") }
            }
            Spacer(Modifier.height(16.dp))
            AndroidView(factory = { PlayerView(it).apply { this.player = player; useController = true } }, modifier = Modifier.fillMaxWidth().weight(1f))
        }
    }
}
