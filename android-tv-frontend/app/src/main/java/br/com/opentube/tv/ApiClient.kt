package br.com.opentube.tv

import org.json.JSONArray
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URI

internal class ApiClient(videoUrl: String) {
    private val base = videoUrl.trimEnd('/')

    fun listVideos(query: String, cursor: String?): VideoPage {
        val params = buildList {
            add("limit=24")
            if (query.isNotBlank()) add("query=${encode(query)}")
            if (!cursor.isNullOrBlank()) add("cursor=${encode(cursor)}")
        }.joinToString("&")
        val json = request("$base/v1/videos?$params")
        val array = json.getJSONArray("items")
        val items = List(array.length()) { index ->
            val item = array.getJSONObject(index)
            VideoSummary(item.getString("id"), item.getString("title"), item.optString("description"), item.optString("creator"), item.getString("playback_url"))
        }
        return VideoPage(items, json.optString("next_cursor").takeIf { it.isNotBlank() && it != "null" })
    }

    private fun request(url: String): JSONObject {
        val connection = URI(url).toURL().openConnection() as HttpURLConnection
        return try {
            connection.connectTimeout = 10_000
            connection.readTimeout = 20_000
            connection.setRequestProperty("Accept", "application/json")
            val code = connection.responseCode
            val stream = if (code in 200..299) connection.inputStream else connection.errorStream
            val text = stream?.bufferedReader()?.use { it.readText() }.orEmpty()
            require(code in 200..299) { "Request failed ($code)." }
            JSONObject(text)
        } finally {
            connection.disconnect()
        }
    }

    private fun encode(value: String) = java.net.URLEncoder.encode(value, Charsets.UTF_8.name())
}

internal val apiClient by lazy { ApiClient(BuildConfig.VIDEO_BASE_URL) }
