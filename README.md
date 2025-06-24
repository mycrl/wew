<!--lint disable no-literal-urls-->
<div align="center">
  <h1>WEW</h1>
</div>
<div align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/mycrl/wew/release.yml?branch=main"/>
  <img src="https://img.shields.io/github/license/mycrl/wew"/>
  <img src="https://img.shields.io/github/issues/mycrl/wew"/>
  <img src="https://img.shields.io/github/stars/mycrl/wew"/>
</div>
<div align="center">
  <sup>
    current version: 
    <a href="https://cef-builds.spotifycdn.com/index.html">cef_binary_137.0.17+gf354b0e+chromium-137.0.7151.104</a>
  </sup>
  </br>
  <sup>platform supported: Windows / macOS</sup>
</div>

---

Wew is a cross-platform WebView rendering library based on [Chromium Embedded Framework (CEF)](https://github.com/chromiumembedded/cef). It supports mouse, keyboard, touch, input methods, off-screen rendering, and communication with web pages.

## Usage

Please note that due to CEF's packaging method, it cannot be integrated with Cargo. The CEF runtime requires many resource files and executables to be placed together, and on macOS, it also needs to follow strict and specific packaging requirements that Cargo cannot handle. Therefore, using it currently requires extensive manual operations or writing custom scripts to automate the process. You can refer to the [windowless_rendering](./examples/windowless_rendering) example to understand how to package your application.

## License

[MIT](./LICENSE) Copyright (c) 2025 Mr.Panda.
