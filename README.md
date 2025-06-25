<!--lint disable no-literal-urls-->
<div align="center">
  <h1>WEW</h1>
</div>
<div align="center">
  <img src="https://img.shields.io/crates/v/wew?style=flat-square"/>
  <img src="https://img.shields.io/docsrs/wew?style=flat-square"/>
  <img src="https://img.shields.io/github/license/mycrl/wew?style=flat-square"/>
  <img src="https://img.shields.io/github/issues/mycrl/wew?style=flat-square"/>
  <img src="https://img.shields.io/github/stars/mycrl/wew?style=flat-square"/>
</div>
<div align="center">
  <sup>
    current cef version: 
    <a href="https://cef-builds.spotifycdn.com/index.html">137.0.17+gf354b0e+chromium-137.0.7151.104</a>
  </sup>
  </br>
  <sup>platform supported: windows / macOS</sup>
</div>

---

Wew is a cross-platform WebView rendering library based on [Chromium Embedded Framework (CEF)](https://github.com/chromiumembedded/cef). It supports mouse, keyboard, touch, input methods, off-screen rendering, and communication with web pages.

## Process Model

> [!NOTE]  
> This project is based on CEF, so the process model is the same as CEF and Chromium. Chromium and CEF (Chromium Embedded Framework) have a very modular process model design, adopting a multi-process architecture primarily to enhance security, stability, and performance. The overall architecture is a multi-process model (Multi-Process Architecture).

#### Browser Process

The main process responsible for managing the entire Chromium/CEF lifecycle. It handles window management, user input (events) processing, network requests (through network services or proxy network processes), scheduling Renderer processes, security policy control, and communication with the system.

#### Renderer Process

Responsible for web page parsing, layout, rendering, JavaScript execution, etc. It executes HTML, CSS, JavaScript, integrates closely with the Blink engine, and each tab/iframe typically corresponds to an independent rendering process (more granular after Site Isolation).

#### GPU Process

Used for hardware-accelerated graphics rendering. It handles WebGL, Canvas, CSS accelerated rendering, etc., and communicates with the operating system's GPU drivers.

#### Utility Processes

Run specific tasks unrelated to web page rendering. Network Service: responsible for all network requests, Audio/Video decode service, database storage, extension processes, etc.

#### Zygote Process (Linux only)

Used for quickly spawning renderer processes to improve performance.

---

Inter-process communication (IPC) is used to synchronize state between Browser and Renderer, implement security controls and permission passing, and transmit events (such as clicks, scrolling, JavaScript calls). Security isolation is ensured by running Renderers in sandboxes, with each website/domain potentially running in an independent Renderer process, and high-privilege operations like GPU and network access not exposed to rendering processes.

## Thread Considerations

In the current project, WebView and Runtime calls are best executed on the UI thread, which is the main thread of the application process.

Creating a Runtime must be completed on the UI thread, and all message loop calls must also be operated on the UI thread.

Other calls should preferably be made on the UI thread unless it's impossible to do so. While they can be operated from any thread, there's currently no guarantee of no other side effects.

## Packaging and Running

> [!NOTE]  
> Please note that due to CEF's packaging method, it cannot be integrated with Cargo. The CEF runtime requires many resource files and executables to be placed together, and on macOS, it also needs to follow strict and specific packaging requirements that Cargo cannot handle. Therefore, using it currently requires extensive manual operations or writing custom scripts to automate the process. You can refer to the ([Windowless Rendering](./examples/windowless_rendering)) example to understand how to package your application.

Let's assume your application name is Kyle.

First, you need to download the CEF preset file. Visit `https://cef-builds.spotifycdn.com/index.html` to download the precompiled file version `cef_binary_137.0.17+gf354b0e+chromium-137.0.7151.104`.

#### Windows

Assuming the executable file is located at `/foo/Kyle.exe`, copy all files from the `cef/Release` and `cef/Resources` directories to the `/foo` directory, placing them alongside the `Kyle.exe` file. However, exclude static library files `.lib` and debug files `.pdb`.

```text
Kyle.exe
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

On macOS, there are more and stricter restrictions, so special attention is needed. First, macOS requires a specific packaging format and needs to separate the subprocess from the main process.

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

For Info.plist, you can modify it according to your own situation. `Chromium Embedded Framework.framework` is the same as on Windows, also coming from the `cef/Release` directory.

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
