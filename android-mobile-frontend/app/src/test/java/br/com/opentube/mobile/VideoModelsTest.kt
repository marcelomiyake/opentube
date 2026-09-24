package br.com.opentube.mobile

import org.junit.Assert.assertEquals
import org.junit.Test

class VideoModelsTest {
    @Test
    fun appendingPagesRemovesDuplicatesAndPreservesOrder() {
        val first = VideoSummary("a", "A", "", "creator", "", "")
        val second = VideoSummary("b", "B", "", "creator", "", "")
        val third = VideoSummary("c", "C", "", "creator", "", "")

        assertEquals(listOf(first, second, third), appendUniqueVideos(listOf(first, second), listOf(second, third)))
    }
}
