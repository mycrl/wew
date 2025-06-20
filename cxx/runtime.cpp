//
//  runtime.cpp
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#ifdef MACOS
#include "include/wrapper/cef_library_loader.h"
#endif

#include "runtime.h"
#include "util.h"

void run_message_loop()
{
    CefRunMessageLoop();
}

void quit_message_loop()
{
    CefQuitMessageLoop();
}

void poll_message_loop()
{
    CefDoMessageLoopWork();
}

void *create_runtime(const RuntimeSettings *settings, RuntimeHandler handler)
{
#ifdef MACOS
    CefScopedLibraryLoader library_loader;
    if (!library_loader.LoadInMain())
    {
        return nullptr;
    }
#endif

    assert(settings != nullptr);

    CefSettings cef_settings;

    CefString(&cef_settings.locale).FromString("en-US");

    cef_settings.no_sandbox = true;
    cef_settings.command_line_args_disabled = true;
    cef_settings.windowless_rendering_enabled = settings->windowless_rendering_enabled;
    cef_settings.multi_threaded_message_loop = settings->multi_threaded_message_loop;
    cef_settings.external_message_pump = settings->external_message_pump;
    cef_settings.background_color = 0xFF;

    if (settings->cache_dir_path != nullptr)
    {
        CefString(&cef_settings.cache_path).FromString(settings->cache_dir_path);
        CefString(&cef_settings.root_cache_path).FromString(settings->cache_dir_path);
    }

    if (settings->browser_subprocess_path != nullptr)
    {
        CefString(&cef_settings.browser_subprocess_path).FromString(settings->browser_subprocess_path);
    }

#ifdef MACOS
    if (settings->framework_dir_path != nullptr)
    {
        CefString(&cef_settings.framework_dir_path).FromString(settings->framework_dir_path);
    }

    if (settings->main_bundle_path != nullptr)
    {
        CefString(&cef_settings.main_bundle_path).FromString(settings->main_bundle_path);
    }
#endif

    Runtime *runtime = new Runtime{new IRuntime(cef_settings, handler)};
    return runtime;
}

void execute_runtime(void *runtime_ptr, int argc, const char **argv)
{
    assert(runtime_ptr != nullptr);

    auto runtime = static_cast<Runtime *>(runtime_ptr);
    auto main_args = get_main_args(argc, argv);
    CefExecuteProcess(main_args, runtime->ref, nullptr);
    CefInitialize(main_args, runtime->ref->GetCefSettings(), runtime->ref, nullptr);
}

void close_runtime(void *runtime_ptr)
{
    assert(runtime_ptr != nullptr);

    CefShutdown();

    delete static_cast<Runtime *>(runtime_ptr);
}

void *create_webview(void *runtime_ptr, const char *url, const WebViewSettings *settings, WebViewHandler handler)
{
    assert(runtime_ptr != nullptr);
    assert(settings != nullptr);
    assert(url != nullptr);

    auto runtime = static_cast<Runtime *>(runtime_ptr);
    auto iwebview = runtime->ref->CreateWebView(std::string(url), settings, handler);
    WebView *webview = new WebView{iwebview};
    return webview;
}

// clang-format off
IRuntime::IRuntime(CefSettings cef_settings, RuntimeHandler handler)
    : _handler(handler)
    , _cef_settings(cef_settings)
{
}
// clang-format on

CefRefPtr<CefBrowserProcessHandler> IRuntime::GetBrowserProcessHandler()
{
    return this;
}

void IRuntime::OnBeforeCommandLineProcessing(const CefString &process_type, CefRefPtr<CefCommandLine> command_line)
{
    command_line->AppendSwitch("use-mock-keychain");
}

void IRuntime::OnContextInitialized()
{
    _handler.on_context_initialized(_handler.context);
}

CefRefPtr<CefClient> IRuntime::GetDefaultClient()
{
    return nullptr;
}

void IRuntime::OnScheduleMessagePumpWork(int64_t delay_ms)
{
    _handler.on_schedule_message_pump_work(delay_ms, _handler.context);
}

CefSettings &IRuntime::GetCefSettings()
{
    return _cef_settings;
}

CefRefPtr<IWebView> IRuntime::CreateWebView(std::string url, const WebViewSettings *settings, WebViewHandler handler)
{
    CefBrowserSettings broswer_settings;
    broswer_settings.webgl = cef_state_t::STATE_DISABLED;
    broswer_settings.databases = cef_state_t::STATE_DISABLED;
    broswer_settings.background_color = 0xFF;

    broswer_settings.default_font_size = settings->default_font_size;
    broswer_settings.windowless_frame_rate = settings->windowless_frame_rate;
    broswer_settings.default_fixed_font_size = settings->default_fixed_font_size;
    broswer_settings.local_storage = settings->local_storage ? STATE_ENABLED : STATE_DISABLED;
    broswer_settings.javascript = settings->javascript ? STATE_ENABLED : STATE_DISABLED;
    broswer_settings.javascript_access_clipboard =
        settings->javascript_access_clipboard ? STATE_ENABLED : STATE_DISABLED;

    CefWindowInfo window_info;
    if (settings->window_handle != nullptr)
    {
        if (_cef_settings.windowless_rendering_enabled)
        {
            window_info.SetAsWindowless((CefWindowHandle)(settings->window_handle));
        }
        else
        {
            CefRect rect(0, 0, settings->width, settings->height);
            window_info.SetAsChild((CefWindowHandle)(settings->window_handle), rect);
        }
    }

    CefRefPtr<IWebView> webview = new IWebView(_cef_settings, settings, handler);
    CefBrowserHost::CreateBrowser(window_info, webview, url, broswer_settings, nullptr, nullptr);
    return webview;
}
