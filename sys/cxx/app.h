//
//  app.h
//  webview
//
//  Created by Mr.Panda on 2023/4/26.
//

#ifndef LIBWEBVIEW_APP_H
#define LIBWEBVIEW_APP_H
#pragma once

#include "bridge.h"
#include "browser.h"
#include "include/cef_app.h"
#include "message_router.h"
#include "webview.h"

class IApp : public CefApp, public CefBrowserProcessHandler
{
public:
    IApp(WebviewOptions* settings, CreateWebviewCallback callback, void* ctx);
    ~IApp()
    {
        router->IClose();
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

    CefRefPtr<IBrowser> CreateBrowser(PageOptions* settings,
                                      PageObserver observer,
                                      void* ctx);

    std::shared_ptr<MessageRouter> router = std::make_shared<MessageRouter>();

private:
    WebviewOptions* _settings;
    CreateWebviewCallback _callback;
    void* _ctx;

    IMPLEMENT_REFCOUNTING(IApp);
};

class IRenderApp : public CefApp, IBridgeHost
{
public:
    /* CefApp */

    void OnRegisterCustomSchemes(CefRawPtr<CefSchemeRegistrar> registrar) override;

    ///
    /// Return the handler for functionality specific to the render process. This
    /// method is called on the render process main thread.
    ///
    CefRefPtr<CefRenderProcessHandler> GetRenderProcessHandler() override;

private:
    IMPLEMENT_REFCOUNTING(IRenderApp);
};

typedef struct
{
    WebviewOptions* settings;
    CefRefPtr<IApp> ref;
} App;

#endif  // LIBWEBVIEW_APP_H
