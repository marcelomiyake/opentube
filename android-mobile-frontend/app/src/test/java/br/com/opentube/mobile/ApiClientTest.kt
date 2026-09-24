package br.com.opentube.mobile

import com.sun.net.httpserver.HttpServer
import java.net.InetSocketAddress
import org.json.JSONObject
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Before
import org.junit.Test

class ApiClientTest {
    private lateinit var server: HttpServer
    private lateinit var client: ApiClient
    private var responseCode = 200
    private var responseBody = ""
    private var requestPath = ""
    private var requestMethod = ""
    private var requestBody = ""
    private var authorization = ""

    @Before
    fun startServer() {
        server = HttpServer.create(InetSocketAddress("127.0.0.1", 0), 0)
        server.createContext("/") { exchange ->
            requestPath = exchange.requestURI.toString()
            requestMethod = exchange.requestMethod
            requestBody = exchange.requestBody.bufferedReader().use { it.readText() }
            authorization = exchange.requestHeaders.getFirst("Authorization").orEmpty()
            val bytes = responseBody.toByteArray()
            if (responseCode == 204) exchange.sendResponseHeaders(responseCode, -1)
            else {
                exchange.sendResponseHeaders(responseCode, bytes.size.toLong())
                exchange.responseBody.use { it.write(bytes) }
            }
        }
        server.start()
        val base = "http://127.0.0.1:${server.address.port}/"
        client = ApiClient(base, base)
    }

    @After
    fun stopServer() {
        server.stop(0)
    }

    @Test
    fun loginPostsCredentialsAndReturnsAccessToken() {
        responseBody = """{"access_token":"signed-token"}"""

        assertEquals("signed-token", client.login("creator@example.test", "secret"))
        assertEquals("POST", requestMethod)
        assertEquals("/v1/login", requestPath)
        assertEquals("creator@example.test", JSONObject(requestBody).getString("email"))
        assertEquals("secret", JSONObject(requestBody).getString("password"))
    }

    @Test
    fun listVideosEncodesSearchCursorAndMapsOptionalFields() {
        responseBody = """{"items":[{"id":"a","title":"Quiet Lake","description":"Morning","creator":"M","published_at":"2026-09-24","playback_url":"http://media/a.m3u8"}],"next_cursor":"next-token"}"""

        val page = client.listVideos("night sky", "cursor / 2")

        assertEquals("/v1/videos?limit=24&query=night+sky&cursor=cursor+%2F+2", requestPath)
        assertEquals("Quiet Lake", page.items.single().title)
        assertEquals("Morning", page.items.single().description)
        assertEquals("http://media/a.m3u8", page.items.single().playbackUrl)
        assertEquals("next-token", page.nextCursor)
    }

    @Test
    fun listVideosOmitsBlankFiltersAndMapsNullCursor() {
        responseBody = """{"items":[],"next_cursor":null}"""

        val page = client.listVideos("  ", " ")

        assertEquals("/v1/videos?limit=24", requestPath)
        assertEquals(emptyList<VideoSummary>(), page.items)
        assertEquals(null, page.nextCursor)
    }

    @Test
    fun createAndCompleteUploadSendCreatorAuthorization() {
        responseBody = """{"id":"video-1","upload_url":"http://media.test/put"}"""
        assertEquals("video-1" to "http://media.test/put", client.createVideo("token", "Title", "Description", 42))
        assertEquals("Bearer token", authorization)
        assertEquals("video/mp4", JSONObject(requestBody).getString("content_type"))
        assertEquals(42, JSONObject(requestBody).getLong("size_bytes"))

        responseCode = 204
        responseBody = ""
        client.completeUpload("token", "video-1")
        assertEquals("POST", requestMethod)
        assertEquals("/v1/videos/video-1/complete", requestPath)
        assertEquals("Bearer token", authorization)
    }

    @Test
    fun reportsHttpErrorsFromTheErrorBody() {
        responseCode = 401
        responseBody = "unauthorized"

        val error = assertThrows(IllegalArgumentException::class.java) {
            client.listVideos("", null)
        }

        assertEquals("Request failed (401). unauthorized", error.message)
    }
}
