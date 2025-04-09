package com.maxgubin.poolcontrol

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import com.maxgubin.poolcontrol.ui.theme.PoolControlTheme

class MainActivity : ComponentActivity() {
    companion object {
        private const val CONFIGURATION_URL = "YOUR_ONLINE_CONFIGURATION_URL_HERE" // Replace with your actual URL
        private const val AUTHENTICATION_TOKEN = "YOUR_BUILT_IN_TOKEN_HERE" // Replace with your actual token
    }

    private lateinit var webView: WebView
    private val coroutineScope = MainScope()


    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContentView(R.layout.activity_main)

        webView = findViewById(R.id.webView)
        webView.settings.javaScriptEnabled = true // Enable JavaScript if needed

        loadWebView("https://www.google.com")
    }

    override fun onDestroy() {
        super.onDestroy()
        coroutineScope.cancel() // Cancel any ongoing coroutines when the activity is destroyed
    }

    private fun loadWebView(webViewUrl: String?) {
        if (!webViewUrl.isNullOrEmpty()) {
            webView.webViewClient = object : WebViewClient() {
                override fun onPageFinished(view: WebView?, url: String?) {
                    // You can perform actions after the page has loaded if needed
                    super.onPageFinished(view, url)
                }
            }

            // Add the authentication token as a header
            val headers = mapOf("Authorization" to "Bearer $AUTHENTICATION_TOKEN") // More idiomatic Kotlin map creation
            webView.loadUrl(webViewUrl, headers)
        } else {
            Toast.makeText(this, "WebView URL not found in configuration.", Toast.LENGTH_LONG).show()
        }
    }
}

