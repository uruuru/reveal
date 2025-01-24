package li.oiu.reveal

import android.content.pm.PackageManager
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject

class MainActivity : TauriActivity() {

  companion object {
    const val STORAGE_PERMISSION_CODE = 100
  }

  var invoke: Invoke? = null

  override fun onRequestPermissionsResult(
    requestCode: Int,
    permissions: Array<out String>,
    grantResults: IntArray
  ) {
    super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        
    // Request for permission happens in RevealPlugin.kt
    if (requestCode == STORAGE_PERMISSION_CODE) {
      val obj = JSObject()
      if (grantResults.isNotEmpty()) {
        obj.put("value", grantResults[0])
        invoke?.resolve(obj)
      } else {
        obj.put("value", PackageManager.PERMISSION_DENIED)
        invoke?.resolve(obj)
      }
      invoke = null
    }
  }
}