package br.com.opentube.tv

import org.junit.Assert.assertEquals
import org.junit.Test

class VideoModelsTest {
    @Test
    fun feedPageAppendKeepsUniqueStableOrder() {
        val a = VideoSummary("a", "A", "", "creator", "")
        val b = VideoSummary("b", "B", "", "creator", "")
        assertEquals(listOf(a, b), appendUniqueVideos(listOf(a), listOf(a, b)))
    }
}
