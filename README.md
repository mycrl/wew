<!--lint disable no-literal-urls-->
<div align="center">
  <h1>webview-rs</h1>
</div>
<br/>
<div align="center">
  <strong>
      <a href="https://github.com/chromiumembedded/cef">Chromium Embedded Framework (CEF)</a>
       bindings for rust.</strong>
</div>
<div align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/mycrl/webview-rs/release.yml?branch=main"/>
  <img src="https://img.shields.io/github/license/mycrl/webview-rs"/>
  <img src="https://img.shields.io/github/issues/mycrl/webview-rs"/>
  <img src="https://img.shields.io/github/stars/mycrl/webview-rs"/>
</div>
<div align="center">
  <sup>
    current version: 
    <a href="https://cef-builds.spotifycdn.com/index.html">cef_binary_137.0.17+gf354b0e+chromium-137.0.7151.104</a>
  </sup>
  </br>
  <sup>platform supported: Windows / Macos</sup>
</div>

---

This is a highly encapsulated CEF library that abstracts common implementations, with significant differences from the original CEF API. It is primarily used to create WebViews based on CEF using Rust, supporting mouse, keyboard, touch, input methods, off-screen rendering, and communication with web pages.

The internal implementation of this project is clean and straightforward, making it easy to develop or add custom modifications. For users who are not satisfied with the current state, customizing it is a simple task.

It's important to note that because CEF's packaging method cannot integrate with cargo, this project is not intended to be published to crates.io. It's recommended to directly depend on the git repository. CEF runtime requires many resource files and executable files to be placed together, and on macOS, it also needs to follow strict and specific packaging methods, which cargo cannot accomplish. Therefore, using it currently requires a lot of manual tasks or writing custom scripts to automate the process. (Here's a reference: https://github.com/mycrl/hylarana/blob/main/applications/app/build.js)

> Note: This project's build script requires cmake. Please ensure cmake exists in your environment variables. You can use `cmake --version` to check if cmake is installed.

## Usage

First, because CEF uses a multi-process model, you need to handle multi-process issues first.

```rust
fn main() {
    if Runtime::is_subprocess() {
        Runtime::execute_subprocess();
        return;
    }
}
```

You need to check at the application entry point whether the current process is a subprocess. If the current process is a subprocess, start running the subprocess. `execute_subprocess` will block until the subprocess closes. If it's not a subprocess, then it's your application's main process, and you can continue executing your own tasks.

Of course, you can separate out a standalone subprocess. Implementing a standalone subprocess will be simpler.

```rust
fn main() {
    Runtime::execute_subprocess();
}
```

Start the subprocess in a separate executable file, which can be separated from the application's main process. This is also a common practice. The subprocess doesn't need to do anything, just `execute_subprocess` in the entry function as shown above. However, note that you also need to pass the path of the subprocess executable file to the `browser_subprocess_path` option of `RuntimeAttributes`.

Then, create a runtime. Note that the runtime is globally unique - you cannot create multiple runtimes. This is undefined behavior, so please don't do this.

```rust
struct RuntimeObserver;

impl RuntimeHandler for RuntimeObserver {
    fn on_context_initialized(&self) {
        println!("Runtime context initialization completed");
    }

    fn on_schedule_message_pump_work(&self, delay: u64) {
        // Do nothing
    }
}

let runtime = RuntimeAttributesBuilder::default()
    .build()
    .create_runtime(RuntimeObserver)?;
```

We create a runtime using default configuration, which defaults to window mode, and all cache data is placed in the current working directory.

The `on_context_initialized` callback in `RuntimeHandler` notifies the external that the runtime has been created successfully, and it's now safe to create WebViews.

`on_schedule_message_pump_work` is macOS-specific, because only macOS doesn't support multi-threaded message loops, so macOS uses a fixed message pump approach. On macOS, you can drive the runtime after the delay specified in this callback, but you can also ignore this callback as long as you can properly drive the runtime in the event loop. However, if you're not on macOS, you don't need to worry about message loop issues at all, because for non-macOS platforms, the runtime internally maintains its own message loop.

Creating WebViews is equally simple. Let's create a WebView to open `google.com`.

```rust
struct WebViewObserver;

impl WebViewHandler for WebViewObserver {
    fn on_state_change(&self, state: WebViewState) {
        println!("WebView state change: {:?}", state);
    }
}

let webview = runtime
    .create_webview("https://google.com", &WebViewAttributes::default(), WebViewObserver)?;
```

Create a WebView from the Runtime. If loading is complete, `on_state_change` on `WebViewHandler` will callback with `WebViewState::Load`.

Using it in window mode is this simple, because the WebView will create its own window, and you don't need to handle interactions between users and the WebView.

#### MacOS

Using this library on macOS will be different, because macOS doesn't support multi-threaded event loops, so you need to manually choose a driving method to make CEF's message loop work properly.

If you're planning to let CEF exclusively use your main thread's message loop, it can be simpler.

```rust
Runtime::block_run();
```

Using `block_run` will block the current thread, and the runtime will use CEF's built-in message loop driver. This function won't return unless you call `Runtime::quit`, which exits the current message loop, and `block_run` will stop blocking and return.

If you already have a message loop and want to embed CEF into it, the usage will be different. Here's an example using a typical `winit` message loop.

```rust
impl ApplicationHandler for App {
    // ... implementation omitted

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        Runtime::poll();
    }
}
```

Here, we drive the message loop pump when winit's `about_to_wait` is triggered, which is a relatively simple approach.

You can also follow CEF's wishes to drive it. The displayed messages generally don't make much difference. In the `on_schedule_message_pump_work` callback mentioned above in `RuntimeHandler`, you can schedule winit's message loop to call `Runtime::poll` after the specified `delay`.

## Packaging and Running

CEF's runtime requires a bunch of resource files and dynamic libraries on Windows, and needs to be packaged into specific directories on macOS. So in a Rust cargo project, it cannot run directly, and the examples included with this project also cannot run directly.

#### Windows

Let's start with the simplest method. This library will automatically compile resource files, so you can find the built resource files in the `./target/debug/build/webview-xxx/out/cef` directory.

Assuming cargo's build directory is `/bar/target/debug/build/webview-4b6671582a188858/out/cef`, and your executable file is at `/foo/webview-example.exe`, copy all files from the `/Release` and `/Resources` directories inside `/bar/target/debug/build/webview-4b6671582a188858/out/cef` to the `/foo` directory, placing them together with `webview-example.exe`, but exclude static libraries `.lib` and debug files `.pdb`.

```text
webview-example.exe
chrome_elf.dll
d3dcompiler_47.dll
dxcompiler.dll
dxil.dll
libcef.dll
libEGL.dll
libGLESv2.dll
v8_context_snapshot.bin
vk_swiftshader_icd.json
vk_swiftshader.dll
vulkan-1.dll
locales
chrome_100_percent.pak
chrome_200_percent.pak
icudtl.dat
resources.pak
```

#### MacOS

On macOS, there are more and stricter restrictions, so special attention is needed. First, macOS requires a specific packaging format and needs to separate the subprocess from the main process. Here we assume the application name is `Kyle`.

```text
Kyle.app
    - Contents
        - Info.plist
        - Frameworks
            - Chromium Embedded Framework.framework
            - Kyle Helper (GPU).app
            - Kyle Helper (Plugin).app
            - Kyle Helper (Renderer).app
            - Kyle Helper.app
        - MacOS
            - Kyle
```

Info.plist:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
    <dict>
        <key>CFBundleName</key>
        <string>Kyle</string>
        <key>CFBundleDisplayName</key>
        <string>Kyle</string>
        <key>CFBundleIdentifier</key>
        <string>com.example.kyle</string>
        <key>CFBundleVersion</key>
        <string>1.0.0</string>
        <key>CFBundleShortVersionString</key>
        <string>1.0</string>
        <key>CFBundlePackageType</key>
        <string>APPL</string>
        <key>CFBundleExecutable</key>
        <string>Kyle</string>
        <key>LSMinimumSystemVersion</key>
        <string>15.4</string>
    </dict>
</plist>
```

For Info.plist, you can modify it according to your own situation. `Chromium Embedded Framework.framework` is the same as on Windows, also coming from the `./target/debug/build/webview-xxx/out/cef/Release` directory.

The several Helpers in Frameworks all need to be generated by yourself. Here's an example using `Kyle Helper (GPU).app`.

```
Kyle Helper (GPU).app
    - Contents
        - Info.plist
        - MacOS
            - Kyle Helper (GPU)
```

Info.plist:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
    <dict>
        <key>CFBundleExecutable</key>
        <string>Kyle Helper (GPU)</string>
        <key>CFBundleIdentifier</key>
        <string>com.github.mycrl.hylarana.helper.gpu</string>
        <key>CFBundlePackageType</key>
        <string>APPL</string>
        <key>CFBundleName</key>
        <string>Kyle Helper (GPU)</string>
    </dict>
</plist>
```

You need to create `Helper (GPU)`, `Helper (Plugin)`, `Helper (Renderer)`, and `Helper` simultaneously. The executable files in these several Helpers are all the same, you just need to change the filename to match the `.app` name. The `Info.plist` also needs to be modified according to the actual situation.

#### Manual Download

Of course, you can also download the CEF's preset files and work with them on your own, rather than looking them up from the cargo target dir. Go to `https://cef-builds.spotifycdn.com/index.html` to download the pre-built files of version `cef_binary_137.0.17+gf354b0e+chromium-137.0.7151.104`.

## Communication with Web Pages

This library's runtime will inject a global object into web pages for communication between Rust and web pages.

```typescript
declare global {
    interface Window {
        MessageTransport: {
            on: (handle: (message: string) => void) => void;
            send: (message: string) => void;
        };
    }
}
```

Usage example:

```typescript
window.MessageTransport.on((message: string) => {
    console.log("Received message from Rust:", message);
});

window.MessageTransport.send("Send message to Rust");
```

`WebViewHandler::on_message` is used to receive messages sent by `MessageTransport.send`, while `MessageTransport.on` is used to receive messages sent by `WebView::send_message`. Sending and receiving messages are full-duplex and asynchronous.

## License

[MIT](./LICENSE) Copyright (c) 2025 Mr.Panda.
