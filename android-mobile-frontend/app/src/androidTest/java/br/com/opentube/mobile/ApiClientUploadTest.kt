package br.com.opentube.mobile

import android.content.ContentValues
import android.os.Environment
import android.provider.MediaStore
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.junit.Assert.assertThrows

@RunWith(AndroidJUnit4::class)
class ApiClientUploadTest {
    @Test
    fun streamsMediaAndReportsStorageRejection() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        val resolver = context.contentResolver
        val values = ContentValues().apply {
            put(MediaStore.MediaColumns.DISPLAY_NAME, "opentube-test-${System.nanoTime()}.mp4")
            put(MediaStore.MediaColumns.MIME_TYPE, "video/mp4")
            put(MediaStore.MediaColumns.RELATIVE_PATH, Environment.DIRECTORY_DOWNLOADS)
        }
        val uri = checkNotNull(resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, values))
        val bytes = "synthetic media fixture".toByteArray()
        try {
            resolver.openOutputStream(uri).use { output -> checkNotNull(output).write(bytes) }
            val client = ApiClient(BuildConfig.IDENTITY_BASE_URL, BuildConfig.VIDEO_BASE_URL)
            val storageUrl = BuildConfig.IDENTITY_BASE_URL

            client.upload(resolver, uri, "$storageUrl/upload", bytes.size.toLong())
            val rejection = assertThrows(IllegalArgumentException::class.java) {
                client.upload(resolver, uri, "$storageUrl/reject", bytes.size.toLong())
            }
            assertTrue(rejection.message.orEmpty().contains("403"))
        } finally {
            resolver.delete(uri, null, null)
        }
    }
}
