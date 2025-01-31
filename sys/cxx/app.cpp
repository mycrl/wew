//
//  app.cpp
//  webview
//
//  Created by Mr.Panda on 2023/4/26.
//

#include "app.h"

#include "include/wrapper/cef_helpers.h"
#include "scheme_handler.h"

IApp::IApp(WebviewOptions* settings, CreateWebviewCallback callback, void* ctx)
    : _settings(settings), _callback(callback), _ctx(ctx)
{
    assert(settings);
}

CefRefPtr<CefBrowserProcessHandler> IApp::GetBrowserProcessHandler()
{
    return this;
}

void IApp::OnContextInitialized()
{
    CEF_REQUIRE_UI_THREAD();

    if (_settings->scheme_path)
    {
        std::string scheme_dir = std::string(_settings->scheme_path);
        RegisterSchemeHandlerFactory(scheme_dir);
    }

    _callback(_ctx);
}

CefRefPtr<CefClient> IApp::GetDefaultClient()
{
    return nullptr;
}

CefRefPtr<IBrowser> IApp::CreateBrowser(PageOptions* settings,
                                        PageObserver observer,
                                        void* ctx)
{
    assert(settings);

    CefBrowserSettings broswer_settings;
    broswer_settings.windowless_frame_rate = settings->frame_rate;
    broswer_settings.webgl = cef_state_t::STATE_DISABLED;
    broswer_settings.background_color = 0x00ffffff;
    broswer_settings.databases = cef_state_t::STATE_DISABLED;

    CefWindowInfo window_info;
    if (settings->is_offscreen)
    {
        window_info.SetAsWindowless((CefWindowHandle)(settings->window_handle));
    }
    else
    {
    #ifdef WIN32
        window_info.SetAsPopup(nullptr, "");
    #endif
    }

    CefRefPtr<IBrowser> browser = new IBrowser(router, settings, observer, ctx);
    CefBrowserHost::CreateBrowser(window_info, browser, settings->url, broswer_settings, nullptr,
                                  nullptr);
    return browser;
}

void IApp::OnRegisterCustomSchemes(CefRawPtr<CefSchemeRegistrar> registrar)
{
    registrar->AddCustomScheme(WEBVIEW_SCHEME_NAME, SCHEME_OPT);
}

CefRefPtr<CefRenderProcessHandler> IRenderApp::GetRenderProcessHandler()
{
    return this;
}

void IRenderApp::OnRegisterCustomSchemes(CefRawPtr<CefSchemeRegistrar> registrar)
{
    registrar->AddCustomScheme(WEBVIEW_SCHEME_NAME, SCHEME_OPT);
}
