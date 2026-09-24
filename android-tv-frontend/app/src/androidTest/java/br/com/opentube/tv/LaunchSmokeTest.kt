package br.com.opentube.tv

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import android.content.Intent
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class LaunchSmokeTest {
    @Test
    fun rendersDpadBrowseAndSearchControls() {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val device = UiDevice.getInstance(instrumentation)
        val launchIntent = Intent(instrumentation.targetContext, MainActivity::class.java)
            .addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK or Intent.FLAG_ACTIVITY_NEW_TASK)

        instrumentation.startActivitySync(launchIntent)

        assertTrue(device.wait(Until.hasObject(By.text("OpenTube")), 20_000))
        assertTrue(device.hasObject(By.text("Search videos")))
        assertTrue(device.hasObject(By.text("Refresh")))
        assertTrue(device.wait(Until.hasObject(By.text("Coverage fixture video")), 10_000))
    }

    @Test
    fun opensFixturePlaybackOrDisplaysFeedConnectionFeedback() {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val device = UiDevice.getInstance(instrumentation)
        val launchIntent = Intent(instrumentation.targetContext, MainActivity::class.java)
            .addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK or Intent.FLAG_ACTIVITY_NEW_TASK)
        instrumentation.startActivitySync(launchIntent)

        assertTrue(device.wait(Until.hasObject(By.text("OpenTube")), 20_000))
        val fixture = By.text("Coverage fixture video")
        assertTrue("the deterministic video stub returns a fixture", device.wait(Until.hasObject(fixture), 10_000))
        device.findObject(fixture).click()
        assertTrue(device.wait(Until.hasObject(By.text("Back")), 10_000))
        device.findObject(By.text("Back")).click()
    }

    @Test
    fun searchShowsAnEmptyFeedMessage() {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val device = UiDevice.getInstance(instrumentation)
        val launchIntent = Intent(instrumentation.targetContext, MainActivity::class.java)
            .addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK or Intent.FLAG_ACTIVITY_NEW_TASK)
        instrumentation.startActivitySync(launchIntent)

        assertTrue(device.wait(Until.hasObject(By.text("Coverage fixture video")), 20_000))
        device.findObject(By.text("Search videos")).click()
        val searchField = device.findObjects(By.clazz("android.widget.EditText")).first()
        searchField.setText("empty")
        device.findObject(By.text("Search")).click()

        assertTrue(device.wait(Until.hasObject(By.text("No videos found. Search or refresh the feed.")), 10_000))
    }

    @Test
    fun searchShowsRetryFeedbackForAnUnavailableApi() {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val device = UiDevice.getInstance(instrumentation)
        val launchIntent = Intent(instrumentation.targetContext, MainActivity::class.java)
            .addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK or Intent.FLAG_ACTIVITY_NEW_TASK)
        instrumentation.startActivitySync(launchIntent)

        assertTrue(device.wait(Until.hasObject(By.text("Coverage fixture video")), 20_000))
        device.findObject(By.text("Search videos")).click()
        val searchField = device.findObjects(By.clazz("android.widget.EditText")).first()
        searchField.setText("failure")
        device.findObject(By.text("Search")).click()

        assertTrue(device.wait(Until.hasObject(By.text("Could not load videos. Try again.")), 10_000))
    }
}
