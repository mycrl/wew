//
//  app.h
//  webview
//
//  Created by Mr.Panda on 2023/4/26.
//

#ifndef LIBWEBVIEW_APP_H
#define LIBWEBVIEW_APP_H
#pragma once

#include "browser.h"
#include "include/cef_app.h"
#include "webview.h"

class IApp : public CefApp, public CefBrowserProcessHandler
{
public:
    IApp(const WebviewOptions* settings, CreateWebviewCallback callback, void* ctx);
    ~IApp()
    {
    }

    /* CefApp */

    void OnRegisterCustomSchemes(CefRawPtr<CefSchemeRegistrar> registrar) override;

    //
    // Return the handler for functionality specific to the browser process. This
    // method is called on multiple threads in the browser process.
    //
    CefRefPtr<CefBrowserProcessHandler> GetBrowserProcessHandler() override;

    /* CefBrowserProcessHandler */

    //
    // Called on the browser process UI thread immediately after the CEF context
    // has been initialized.
    //
    void OnContextInitialized() override;

    //
    // Return the default client for use with a newly created browser window. If
    // null is returned the browser will be unmanaged (no callbacks will be
    // executed for that browser) and application shutdown will be blocked until
    // the browser window is closed manually. This method is currently only used
    // with the chrome runtime.
    //
    CefRefPtr<CefClient> GetDefaultClient() override;

    CefRefPtr<IBrowser> CreateBrowser(std::string url,
                                      const PageOptions* settings,
                                      PageObserver observer,
                                      void* ctx);

    CefSettings cef_settings;
private:
    std::optional<std::string> _scheme_path = std::nullopt;
    CreateWebviewCallback _callback;
    void* _ctx;

    IMPLEMENT_REFCOUNTING(IApp);
};

class MessageSendFunction : public CefV8Handler
{
public:
    MessageSendFunction()
    {
    }

    /* CefV8Handler */

    bool Execute(const CefString& name,
                 CefRefPtr<CefV8Value> object,
                 const CefV8ValueList& arguments,
                 CefRefPtr<CefV8Value>& retval,
                 CefString& exception);

    void SetBrowser(CefRefPtr<CefBrowser> browser)
    {
        _browser = std::optional(browser);
    }
private:
    std::optional<CefRefPtr<CefBrowser>> _browser = std::nullopt;

    IMPLEMENT_REFCOUNTING(MessageSendFunction);
};

class MessageOnFunction : public CefV8Handler
{
public:
    MessageOnFunction()
    {
    }

    /* CefV8Handler */

    bool Execute(const CefString& name,
                 CefRefPtr<CefV8Value> object,
                 const CefV8ValueList& arguments,
                 CefRefPtr<CefV8Value>& retval,
                 CefString& exception);

    void Call(std::string message);
private:
    std::optional<CefRefPtr<CefV8Context>> _context = std::nullopt;
    std::optional<CefRefPtr<CefV8Value>> _callback = std::nullopt;

    IMPLEMENT_REFCOUNTING(MessageOnFunction);
};

class IRenderApp : public CefApp, public CefRenderProcessHandler
{
public:
    /* CefApp */

    void OnRegisterCustomSchemes(CefRawPtr<CefSchemeRegistrar> registrar) override;

    ///
    /// Return the handler for functionality specific to the render process. This
    /// method is called on the render process main thread.
    ///
    CefRefPtr<CefRenderProcessHandler> GetRenderProcessHandler() override;

    /* CefRenderProcessHandler */

    void OnContextCreated(CefRefPtr<CefBrowser> browser,
                          CefRefPtr<CefFrame> frame,
                          CefRefPtr<CefV8Context> context);
    bool OnProcessMessageReceived(CefRefPtr<CefBrowser> browser,
                                  CefRefPtr<CefFrame> frame,
                                  CefProcessId source_process,
                                  CefRefPtr<CefProcessMessage> message);

private:
    CefRefPtr<MessageSendFunction> _send_func = new MessageSendFunction();
    CefRefPtr<MessageOnFunction> _on_func = new MessageOnFunction();

    IMPLEMENT_REFCOUNTING(IRenderApp);
};

typedef struct
{
    CefRefPtr<IApp> ref;
} App;

#endif  // LIBWEBVIEW_APP_H
