package br.com.opentube.mobile

data class VideoSummary(
    val id: String,
    val title: String,
    val description: String,
    val creator: String,
    val publishedAt: String,
    val playbackUrl: String,
)

data class VideoPage(val items: List<VideoSummary>, val nextCursor: String?)

fun appendUniqueVideos(current: List<VideoSummary>, next: List<VideoSummary>): List<VideoSummary> {
    val seen = current.mapTo(mutableSetOf()) { it.id }
    return current + next.filter { seen.add(it.id) }
}
