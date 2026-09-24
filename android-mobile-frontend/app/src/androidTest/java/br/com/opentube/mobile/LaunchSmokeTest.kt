package br.com.opentube.mobile

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
    fun rendersMobileFeedAndCreatorSignIn() {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val device = UiDevice.getInstance(instrumentation)
        val launchIntent = Intent(instrumentation.targetContext, MainActivity::class.java)
            .addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK or Intent.FLAG_ACTIVITY_NEW_TASK)

        instrumentation.startActivitySync(launchIntent)

        assertTrue(device.wait(Until.hasObject(By.text("OpenTube")), 20_000))
        assertTrue(device.hasObject(By.text("Search videos")))
        assertTrue(device.hasObject(By.text("Creator sign-in")))
        val fixture = By.text("Coverage fixture video")
        assertTrue("the deterministic video stub returns a fixture", device.wait(Until.hasObject(fixture), 10_000))
        device.findObject(fixture).click()
        assertTrue(device.wait(Until.hasObject(By.text("Close")), 10_000))
        device.findObject(By.text("Close")).click()
    }

    @Test
    fun searchShowsAnEmptyFeedMessage() {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val device = UiDevice.getInstance(instrumentation)
        val launchIntent = Intent(instrumentation.targetContext, MainActivity::class.java)
            .addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK or Intent.FLAG_ACTIVITY_NEW_TASK)
        instrumentation.startActivitySync(launchIntent)

        assertTrue(device.wait(Until.hasObject(By.text("Coverage fixture video")), 20_000))
        device.findObjects(By.clazz("android.widget.EditText")).first().setText("empty")
        device.findObject(By.text("Search")).click()

        assertTrue(
            device.wait(
                Until.hasObject(By.text("No videos yet. Upload one from the phone or web client.")),
                10_000,
            ),
        )
    }

    @Test
    fun searchShowsRetryFeedbackForAnUnavailableApi() {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val device = UiDevice.getInstance(instrumentation)
        val launchIntent = Intent(instrumentation.targetContext, MainActivity::class.java)
            .addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK or Intent.FLAG_ACTIVITY_NEW_TASK)
        instrumentation.startActivitySync(launchIntent)

        assertTrue(device.wait(Until.hasObject(By.text("Coverage fixture video")), 20_000))
        device.findObjects(By.clazz("android.widget.EditText")).first().setText("failure")
        device.findObject(By.text("Search")).click()

        assertTrue(
            device.wait(
                Until.hasObject(By.text("Could not load videos. Pull down and try again.")),
                10_000,
            ),
        )
    }

    @Test
    fun reportsSignInAndUploadValidationFeedback() {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val context = instrumentation.targetContext
        val preferences = context.getSharedPreferences("opentube", 0)
        preferences.edit().clear().commit()
        val device = UiDevice.getInstance(instrumentation)
        val launchIntent = Intent(context, MainActivity::class.java)
            .addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK or Intent.FLAG_ACTIVITY_NEW_TASK)
        instrumentation.startActivitySync(launchIntent)

        assertTrue(device.wait(Until.hasObject(By.text("OpenTube")), 20_000))
        val signInButtons = device.findObjects(By.text("Sign in"))
        assertTrue("the seeded creator panel has a sign-in action", signInButtons.size >= 2)
        signInButtons.last().click()
        assertTrue(device.wait(Until.hasObject(By.text("Sign-in failed. Check the local demo credentials.")), 20_000))

        preferences.edit().putString("token", "expired-test-token").commit()
        instrumentation.startActivitySync(launchIntent)
        assertTrue(device.wait(Until.hasObject(By.text("Video title")), 20_000))
        device.findObject(By.text("Upload")).click()
        assertTrue(device.wait(Until.hasObject(By.text("Add a title and choose an MP4 first.")), 5_000))
        preferences.edit().clear().commit()
    }

    @Test
    fun signsInAndSignsOutFromTheSeededCreatorPanel() {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val context = instrumentation.targetContext
        val preferences = context.getSharedPreferences("opentube", 0)
        preferences.edit().clear().commit()
        val device = UiDevice.getInstance(instrumentation)
        val launchIntent = Intent(context, MainActivity::class.java)
            .addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK or Intent.FLAG_ACTIVITY_NEW_TASK)
        instrumentation.startActivitySync(launchIntent)

        assertTrue(device.wait(Until.hasObject(By.text("Creator sign-in")), 20_000))
        val fields = device.findObjects(By.clazz("android.widget.EditText"))
        assertTrue("the creator form has a password field", fields.size >= 2)
        fields.last().setText("synthetic-test-password")
        device.findObjects(By.text("Sign in")).last().click()

        assertTrue(device.wait(Until.hasObject(By.text("Signed in as creator.")), 20_000))
        assertTrue(device.hasObject(By.text("Video title")))
        device.findObject(By.text("Sign out")).click()
        assertTrue(device.wait(Until.hasObject(By.text("Creator sign-in")), 5_000))
        preferences.edit().clear().commit()
    }
}
