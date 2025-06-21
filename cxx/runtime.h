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
#include "scheme.h"
#include "webview.h"

class IRuntime : public CefApp, public CefBrowserProcessHandler
{
  public:
    IRuntime(const RuntimeSettings *settings, CefSettings cef_settings, RuntimeHandler handler);
    ~IRuntime()
    {
    }

    /* CefApp */

    virtual void OnRegisterCustomSchemes(CefRawPtr<CefSchemeRegistrar> registrar) override;

    ///
    /// Return the handler for functionality specific to the browser process. This
    /// method is called on multiple threads in the browser process.
    ///
    virtual CefRefPtr<CefBrowserProcessHandler> GetBrowserProcessHandler() override;

    ///
    /// Provides an opportunity to view and/or modify command-line arguments
    /// before processing by CEF and Chromium. The |process_type| value will be
    /// empty for the browser process. Do not keep a reference to the
    /// CefCommandLine object passed to this method. The
    /// cef_settings_t.command_line_args_disabled value can be used to start with
    /// an empty command-line object. Any values specified in CefSettings that
    /// equate to command-line arguments will be set before this method is called.
    /// Be cautious when using this method to modify command-line arguments for
    /// non-browser processes as this may result in undefined behavior including
    /// crashes.
    ///
    virtual void OnBeforeCommandLineProcessing(const CefString &process_type,
                                               CefRefPtr<CefCommandLine> command_line) override;

    /* CefBrowserProcessHandler */

    ///
    /// Called on the browser process UI thread immediately after the CEF context
    /// has been initialized.
    ///
    virtual void OnContextInitialized() override;

    ///
    /// Return the default client for use with a newly created browser window
    /// (CefBrowser object). If null is returned the CefBrowser will be unmanaged
    /// (no callbacks will be executed for that CefBrowser) and application
    /// shutdown will be blocked until the browser window is closed manually. This
    /// method is currently only used with Chrome style when creating new browser
    /// windows via Chrome UI.
    ///
    virtual CefRefPtr<CefClient> GetDefaultClient() override;

    ///
    /// Called from any thread when work has been scheduled for the browser
    /// process main (UI) thread. This callback is used in combination with
    /// cef_settings_t.external_message_pump and CefDoMessageLoopWork() in cases
    /// where the CEF message loop must be integrated into an existing application
    /// message loop (see additional comments and warnings on
    /// CefDoMessageLoopWork). This callback should schedule a
    /// CefDoMessageLoopWork() call to happen on the main (UI) thread. |delay_ms|
    /// is the requested delay in milliseconds. If |delay_ms| is <= 0 then the
    /// call should happen reasonably soon. If |delay_ms| is > 0 then the call
    /// should be scheduled to happen after the specified delay and any currently
    /// pending scheduled call should be cancelled.
    ///
    virtual void OnScheduleMessagePumpWork(int64_t delay_ms) override;

    CefRefPtr<IWebView> CreateWebView(std::string url, const WebViewSettings *settings, WebViewHandler handler);
    CefSettings &GetCefSettings();

  private:
    std::optional<ICustomSchemeAttributes> _custom_scheme;
    CefSettings _cef_settings;
    RuntimeHandler _handler;

    IMPLEMENT_REFCOUNTING(IRuntime);
};

typedef struct
{
    CefRefPtr<IRuntime> ref;
} Runtime;

#endif /* runtime_h */
