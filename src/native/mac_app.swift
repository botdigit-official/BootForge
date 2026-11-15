import Cocoa
import WebKit

class AppDelegate: NSObject, NSApplicationDelegate, NSWindowDelegate, WKNavigationDelegate {
    var window: NSWindow!
    var webView: WKWebView!
    var backendProcess: Process?
    let defaultPort: UInt16 = 4242

    func applicationDidFinishLaunching(_ notification: Notification) {
        startBackendServer()
        setupNativeWindow()
        loadWebApp()
    }

    func startBackendServer() {
        let bundlePath = Bundle.main.bundlePath
        let binaryPath = (bundlePath as NSString).appendingPathComponent("Contents/MacOS/bootforge_engine")
        
        let pathToCheck = FileManager.default.fileExists(atPath: binaryPath) ? binaryPath : "./target/release/bootforge"
        
        if FileManager.default.fileExists(atPath: pathToCheck) {
            let proc = Process()
            proc.executableURL = URL(fileURLWithPath: pathToCheck)
            proc.arguments = ["ui", "--port", "\(defaultPort)", "--no-open"]
            
            // Redirect output to log file for easy debugging
            let logPath = "/tmp/bootforge_engine.log"
            FileManager.default.createFile(atPath: logPath, contents: nil, attributes: nil)
            if let fileHandle = FileHandle(forWritingAtPath: logPath) {
                proc.standardOutput = fileHandle
                proc.standardError = fileHandle
            }
            
            do {
                try proc.run()
                self.backendProcess = proc
                NSLog("BootForge: backend engine process started (PID %d)", proc.processIdentifier)
            } catch {
                NSLog("BootForge: failed to start backend process: %@", error.localizedDescription)
            }
        } else {
            NSLog("BootForge: binary path not found: %@", pathToCheck)
        }
    }

    func setupNativeWindow() {
        let windowWidth: CGFloat = 1280
        let windowHeight: CGFloat = 840
        let rect = NSRect(x: 100, y: 100, width: windowWidth, height: windowHeight)
        
        window = NSWindow(
            contentRect: rect,
            styleMask: [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        window.title = "Botdigit BootForge"
        window.titlebarAppearsTransparent = true
        window.titleVisibility = .hidden
        window.isMovableByWindowBackground = true
        window.center()
        window.minSize = NSSize(width: 980, height: 680)
        window.appearance = NSAppearance(named: .darkAqua)
        window.delegate = self
        
        let config = WKWebViewConfiguration()
        config.preferences.setValue(true, forKey: "developerExtrasEnabled")
        
        webView = WKWebView(frame: rect, configuration: config)
        webView.autoresizingMask = [.width, .height]
        webView.setValue(false, forKey: "drawsBackground") // Transparent background until loaded
        webView.navigationDelegate = self
        
        window.contentView = webView
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    func loadWebApp() {
        // Poll for server readiness
        let url = URL(string: "http://127.0.0.1:\(defaultPort)")!
        var attempts = 0
        Timer.scheduledTimer(withTimeInterval: 0.2, repeats: true) { [weak self] timer in
            guard let self = self else { timer.invalidate(); return }
            attempts += 1
            var request = URLRequest(url: url)
            request.timeoutInterval = 0.5
            
            let task = URLSession.shared.dataTask(with: request) { _, response, _ in
                if let httpResp = response as? HTTPURLResponse, httpResp.statusCode == 200 {
                    DispatchQueue.main.async {
                        NSLog("BootForge: server ready at %@", url.absoluteString)
                        self.webView.load(URLRequest(url: url))
                        timer.invalidate()
                    }
                }
            }
            task.resume()
            
            if attempts > 50 {
                // Fallback direct load
                DispatchQueue.main.async {
                    NSLog("BootForge: fallback direct load to %@", url.absoluteString)
                    self.webView.load(URLRequest(url: url))
                    timer.invalidate()
                }
            }
        }
    }

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        NSLog("BootForge: webView finished loading")
    }

    func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
        NSLog("BootForge: webView navigation failed: %@", error.localizedDescription)
    }

    func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) {
        NSLog("BootForge: webView provisional navigation failed: %@", error.localizedDescription)
    }

    func applicationWillTerminate(_ notification: Notification) {
        backendProcess?.terminate()
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        return true
    }
}

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.run()
