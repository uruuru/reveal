package li.oiu.reveal

import android.Manifest
import android.app.Activity
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.webkit.MimeTypeMap
import androidx.appcompat.app.AlertDialog
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
class UrlArgs {
  var value: String? = null
}

@TauriPlugin
class RevealPlugin(private val activity: Activity) : Plugin(activity) {

  // TODO check if it makes sense and things easier to use
  //   checkPermissions()
  //   requestAllPermissions()
  // defined in Plugin.kt alongside @TauriPlugin(permissions = [...])

  @Command
  fun checkAndRequestPermissions(invoke: Invoke) {

    val permission = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      Manifest.permission.READ_MEDIA_IMAGES
    } else {
      Manifest.permission.READ_EXTERNAL_STORAGE
    }

    val permissionState = ContextCompat.checkSelfPermission(activity, permission)
    if (permissionState != PackageManager.PERMISSION_GRANTED) {
      // Hacky way to pass the callback
      (activity as MainActivity).invoke = invoke

      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
        AlertDialog.Builder(activity)
          .setTitle("Access to images")
          .setMessage("Please provide access to your images such that we can show them to you 😉.")
          .setPositiveButton("Ok") { _, _ ->

            // TODO does not show the android dialog again if a user rejected once
            // Use shouldShowRequestPermissionRationale or mention to the user to go to settings herself
            ActivityCompat.requestPermissions(
              activity,
              arrayOf(permission),
              MainActivity.STORAGE_PERMISSION_CODE
            )
          }
          .create()
          .show()

      } else {
        // For Android 5.1 and below
        ActivityCompat.requestPermissions(
          activity,
          arrayOf(permission),
          MainActivity.STORAGE_PERMISSION_CODE
        )
      }
    } else {
      val obj = JSObject()
      obj.put("value", permissionState)
      invoke.resolve(obj)
    }
  }

  @Command
  fun getMimeType(invoke: Invoke) {
    val args = invoke.parseArgs(UrlArgs::class.java)
    val uri = Uri.parse(args.value)
    
    val mimeType = activity.contentResolver.getType(uri)
    val fileExtension = MimeTypeMap.getSingleton().getExtensionFromMimeType(mimeType)

    val obj = JSObject()
    obj.put("value", fileExtension)
    invoke.resolve(obj)
  }

}