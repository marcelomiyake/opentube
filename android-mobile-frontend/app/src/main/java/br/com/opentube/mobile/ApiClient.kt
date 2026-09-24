package br.com.opentube.mobile

import android.content.ContentResolver
import android.net.Uri
import org.json.JSONArray
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URI

internal class ApiClient(identityUrl: String, videoUrl: String) {
    private val identityBase = identityUrl.trimEnd('/')
    private val videoBase = videoUrl.trimEnd('/')

    fun login(email: String, password: String): String {
        val body = JSONObject().put("email", email).put("password", password)
        return requestJson("$identityBase/v1/login", "POST", body).getString("access_token")
    }

    fun listVideos(query: String, cursor: String?): VideoPage {
        val params = buildList {
            add("limit=24")
            if (query.isNotBlank()) add("query=${encode(query)}")
            if (!cursor.isNullOrBlank()) add("cursor=${encode(cursor)}")
        }.joinToString("&")
        val json = requestJson("$videoBase/v1/videos?$params", "GET")
        val items = json.getJSONArray("items").toVideos()
        return VideoPage(items, json.optString("next_cursor").takeIf { it.isNotBlank() && it != "null" })
    }

    fun createVideo(token: String, title: String, description: String, sizeBytes: Long): Pair<String, String> {
        val body = JSONObject()
            .put("title", title)
            .put("description", description)
            .put("content_type", "video/mp4")
            .put("size_bytes", sizeBytes)
        val response = requestJson("$videoBase/v1/videos", "POST", body, token)
        return response.getString("id") to response.getString("upload_url")
    }

    fun upload(resolver: ContentResolver, uri: Uri, url: String, sizeBytes: Long) {
        val connection = (URI(url).toURL().openConnection() as HttpURLConnection).apply {
            requestMethod = "PUT"
            doOutput = true
            connectTimeout = 30_000
            readTimeout = 60_000
            setRequestProperty("Content-Type", "video/mp4")
            setFixedLengthStreamingMode(sizeBytes)
        }
        try {
            val input = resolver.openInputStream(uri) ?: error("The selected file cannot be opened.")
            input.use { source -> connection.outputStream.use { output -> source.copyTo(output) } }
            require(connection.responseCode in 200..299) { "Media storage rejected the upload (${connection.responseCode})." }
        } finally {
            connection.disconnect()
        }
    }

    fun completeUpload(token: String, videoId: String) {
        requestJson("$videoBase/v1/videos/$videoId/complete", "POST", JSONObject(), token)
    }

    private fun requestJson(url: String, method: String, body: JSONObject? = null, token: String? = null): JSONObject {
        val connection = (URI(url).toURL().openConnection() as HttpURLConnection).apply {
            requestMethod = method
            connectTimeout = 10_000
            readTimeout = 20_000
            setRequestProperty("Accept", "application/json")
            if (body != null) {
                doOutput = true
                setRequestProperty("Content-Type", "application/json")
            }
            if (token != null) setRequestProperty("Authorization", "Bearer $token")
        }
        try {
            if (body != null) connection.outputStream.bufferedWriter().use { it.write(body.toString()) }
            val code = connection.responseCode
            val stream = if (code in 200..299) connection.inputStream else connection.errorStream
            val text = stream?.bufferedReader()?.use { it.readText() }.orEmpty()
            require(code in 200..299) { "Request failed ($code). ${text.take(160)}" }
            return if (text.isBlank()) JSONObject() else JSONObject(text)
        } finally {
            connection.disconnect()
        }
    }

    private fun JSONArray.toVideos(): List<VideoSummary> = List(length()) { index ->
        val item = getJSONObject(index)
        VideoSummary(
            id = item.getString("id"),
            title = item.getString("title"),
            description = item.optString("description"),
            creator = item.optString("creator"),
            publishedAt = item.optString("published_at"),
            playbackUrl = item.getString("playback_url"),
        )
    }

    private fun encode(value: String) = java.net.URLEncoder.encode(value, Charsets.UTF_8.name())
}

internal val apiClient by lazy { ApiClient(BuildConfig.IDENTITY_BASE_URL, BuildConfig.VIDEO_BASE_URL) }
