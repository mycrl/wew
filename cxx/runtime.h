//
//  runtime.h
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#ifndef runtime_h
#define runtime_h
#pragma once

#include <optional>
#include <string>

#include "include/cef_app.h"

#include "library.h"
#include "request.h"
#include "webview.h"

class IRuntime : public CefApp, public CefBrowserProcessHandler
{
  public:
    IRuntime(const RuntimeSettings *settings, CefSettings cef_settings, RuntimeHandler handler);
    ~IRuntime()
    {
    }

    /* CefApp */

    ///
    /// Provides an opportunity to register custom schemes.
    ///
    void OnRegisterCustomSchemes(CefRawPtr<CefSchemeRegistrar> registrar) override;

    ///
    /// Return the handler for functionality specific to the browser process.
    ///
    /// This method is called on multiple threads in the browser process.
    ///
    CefRefPtr<CefBrowserProcessHandler> GetBrowserProcessHandler() override;

    ///
    /// Provides an opportunity to view and/or modify command-line arguments before processing by CEF and Chromium.
    ///
    void OnBeforeCommandLineProcessing(const CefString &process_type, CefRefPtr<CefCommandLine> command_line) override;

    /* CefBrowserProcessHandler */

    ///
    /// Called on the browser process UI thread immediately after the CEF context has been initialized.
    ///
    void OnContextInitialized() override;

    ///
    /// Return the default client for use with a newly created browser window (CefBrowser object).
    ///
    CefRefPtr<CefClient> GetDefaultClient() override;

    ///
    /// Called from any thread when work has been scheduled for the browser process main (UI) thread.
    ///
    void OnScheduleMessagePumpWork(int64_t delay_ms) override;

    ///
    /// Called before a child process is launched.
    ///
    void OnBeforeChildProcessLaunch(CefRefPtr<CefCommandLine> command_line) override;

    CefRefPtr<IWebView> CreateWebView(std::string url, const WebViewSettings *settings, WebViewHandler handler);
    CefSettings &GetCefSettings();

  private:
    std::optional<ICustomSchemeAttributes> _custom_scheme = std::nullopt;
    CefSettings _cef_settings;
    RuntimeHandler _handler;

    IMPLEMENT_REFCOUNTING(IRuntime);
};

typedef struct
{
    CefRefPtr<IRuntime> ref;
} Runtime;

#endif /* runtime_h */
