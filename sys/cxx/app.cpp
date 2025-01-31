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
    : _callback(callback), _ctx(ctx)
{
    assert(settings);

    cef_settings.windowless_rendering_enabled = true;
    cef_settings.chrome_runtime = false;
    cef_settings.no_sandbox = true;
    cef_settings.background_color = 0x00ffffff;

    // macos not support the multi threaded message loop.
#ifdef MACOS
    cef_settings.multi_threaded_message_loop = false;
#else
    cef_settings.multi_threaded_message_loop = true;
#endif

    CefString(&cef_settings.locale).FromString("zh-CN");

    if (settings->cache_path != nullptr)
    {
        CefString(&cef_settings.cache_path).FromString(settings->cache_path);
        CefString(&cef_settings.log_file).FromString(std::string(settings->cache_path) + "/webview.log");
    }

    if (settings->browser_subprocess_path != nullptr)
    {
        CefString(&cef_settings.browser_subprocess_path).FromString(settings->browser_subprocess_path);
    }

    if (settings->scheme_path != nullptr)
    {
        _scheme_path = std::string(settings->scheme_path);
    }
}

CefRefPtr<CefBrowserProcessHandler> IApp::GetBrowserProcessHandler()
{
    return this;
}

void IApp::OnContextInitialized()
{
    CEF_REQUIRE_UI_THREAD();

    if (_scheme_path.has_value())
    {
        RegisterSchemeHandlerFactory(_scheme_path.value());
    }

    _callback(_ctx);
}

CefRefPtr<CefClient> IApp::GetDefaultClient()
{
    return nullptr;
}

CefRefPtr<IBrowser> IApp::CreateBrowser(std::string url, 
                                        PageOptions* settings_ptr,
                                        PageObserver observer,
                                        void* ctx)
{
    assert(settings_ptr);

    PageOptions settings;
    memcpy(&settings, settings_ptr, sizeof(PageOptions));

    CefBrowserSettings broswer_settings;
    broswer_settings.windowless_frame_rate = settings.frame_rate;
    broswer_settings.webgl = cef_state_t::STATE_DISABLED;
    broswer_settings.background_color = 0x00ffffff;
    broswer_settings.databases = cef_state_t::STATE_DISABLED;

    CefWindowInfo window_info;
    window_info.bounds.width = settings.width;
    window_info.bounds.height = settings.height;

    if (settings.is_offscreen)
    {
        window_info.SetAsWindowless((CefWindowHandle)(settings.window_handle));
    }

    CefRefPtr<IBrowser> browser = new IBrowser(router, settings, observer, ctx);
    CefBrowserHost::CreateBrowser(window_info, browser, url, broswer_settings, nullptr, nullptr);
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
