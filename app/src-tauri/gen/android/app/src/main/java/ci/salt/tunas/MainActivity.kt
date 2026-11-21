package ci.salt.tunas

import android.content.Intent
import android.os.Bundle
import android.view.View
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge
import androidx.core.content.FileProvider
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import java.io.File

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    // Keep TUNAS clear of the status bar, the navigation bar, a camera cut-out and the on-screen keyboard.
    val root = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(root) { v, insets ->
      val bars = insets.getInsets(
        WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout() or WindowInsetsCompat.Type.ime()
      )
      v.setPadding(bars.left, bars.top, bars.right, bars.bottom)
      WindowInsetsCompat.CONSUMED
    }
  }

  override fun onWebViewCreate(webView: WebView) {
    webView.addJavascriptInterface(TunasAndroid(), "TunasAndroid")
  }

  /** Called from TUNAS's update card: open Android's install screen for the APK TUNAS downloaded and checked. */
  inner class TunasAndroid {
    @JavascriptInterface
    fun installUpdate(path: String): String {
      return try {
        val updates = File(filesDir, "TUNAS/updates").canonicalFile
        val file = File(path).canonicalFile
        if (file.parentFile != updates || !file.name.endsWith(".apk") || !file.exists()) return "Not a TUNAS update."
        val uri = FileProvider.getUriForFile(this@MainActivity, "$packageName.fileprovider", file)
        val intent = Intent(Intent.ACTION_VIEW).apply {
          setDataAndType(uri, "application/vnd.android.package-archive")
          addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_ACTIVITY_NEW_TASK)
        }
        runOnUiThread { startActivity(intent) }
        ""
      } catch (e: Exception) {
        e.message ?: "Could not open the installer."
      }
    }
  }
}
