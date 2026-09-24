package br.com.opentube.tv

import com.sun.net.httpserver.HttpServer
import java.net.InetSocketAddress
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

    @Before
    fun startServer() {
        server = HttpServer.create(InetSocketAddress("127.0.0.1", 0), 0)
        server.createContext("/") { exchange ->
            requestPath = exchange.requestURI.toString()
            val bytes = responseBody.toByteArray()
            exchange.sendResponseHeaders(responseCode, bytes.size.toLong())
            exchange.responseBody.use { it.write(bytes) }
        }
        server.start()
        client = ApiClient("http://127.0.0.1:${server.address.port}/")
    }

    @After
    fun stopServer() {
        server.stop(0)
    }

    @Test
    fun mapsVideosAndEncodesSearchAndCursor() {
        responseBody = """{"items":[{"id":"tv-1","title":"Night Sky","description":"Stars","creator":"M","playback_url":"http://media/tv.m3u8"}],"next_cursor":"after"}"""

        val page = client.listVideos("night sky", "cursor / 2")

        assertEquals("/v1/videos?limit=24&query=night+sky&cursor=cursor+%2F+2", requestPath)
        assertEquals(VideoSummary("tv-1", "Night Sky", "Stars", "M", "http://media/tv.m3u8"), page.items.single())
        assertEquals("after", page.nextCursor)
    }

    @Test
    fun handlesEmptyFeedAndHttpErrors() {
        responseBody = """{"items":[],"next_cursor":"null"}"""
        val page = client.listVideos("", null)
        assertEquals("/v1/videos?limit=24", requestPath)
        assertEquals(emptyList<VideoSummary>(), page.items)
        assertEquals(null, page.nextCursor)

        responseCode = 503
        responseBody = "unavailable"
        val error = assertThrows(IllegalArgumentException::class.java) { client.listVideos("", null) }
        assertEquals("Request failed (503).", error.message)
    }
}
