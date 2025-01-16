package li.oiu.reveal

import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat

class MainActivity : TauriActivity() {
    
    private val STORAGE_PERMISSION_CODE = 100
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val permission = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            Manifest.permission.READ_MEDIA_IMAGES
        } else {
            Manifest.permission.READ_EXTERNAL_STORAGE
        }

        if (ContextCompat.checkSelfPermission(this, permission) != PackageManager.PERMISSION_GRANTED) {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                AlertDialog.Builder(this)
                    .setTitle("Access to images")
                    .setMessage("The app needs access to your images.")
                    .setPositiveButton("Ok") { _, _ ->
                        ActivityCompat.requestPermissions(
                            this,
                            arrayOf(permission),
                            STORAGE_PERMISSION_CODE
                        )
                    }
                    .create()
                    .show()
            } else {
                // For Android 5.1 and below
                ActivityCompat.requestPermissions(
                    this,
                    arrayOf(permission),
                    STORAGE_PERMISSION_CODE
                )
            }
        }
    }
    
    // override fun onRequestPermissionsResult(requestCode: Int, permissions: Array<out String>, grantResults: IntArray) {
    //     super.onRequestPermissionsResult(requestCode, permissions, grantResults)
    //     if (requestCode == STORAGE_PERMISSION_CODE && grantResults.isNotEmpty()) {
    //         if (grantResults[0] == PackageManager.PERMISSION_GRANTED) {
    //             // Nothing at the moment.
    //         }
    //     }
    // }
}