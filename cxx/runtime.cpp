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

// clang-format off
IRuntime::IRuntime(const RuntimeSettings *settings, CefSettings cef_settings, RuntimeHandler handler)
    : _handler(handler)
    , _cef_settings(cef_settings)
{
    if (settings->custom_scheme != nullptr)
    {
        assert(settings->custom_scheme->factory != nullptr);

        _custom_scheme = ICustomSchemeAttributes{
            .name = std::string(settings->custom_scheme->name),
            .domain = std::string(settings->custom_scheme->domain),
            .factory = settings->custom_scheme->factory,
        };
    }
}
// clang-format on

CefRefPtr<CefBrowserProcessHandler> IRuntime::GetBrowserProcessHandler()
{
    return this;
}

void IRuntime::OnRegisterCustomSchemes(CefRawPtr<CefSchemeRegistrar> registrar)
{
    if (_custom_scheme.has_value())
    {
        registrar->AddCustomScheme(_custom_scheme.value().name,
                                   CEF_SCHEME_OPTION_STANDARD | CEF_SCHEME_OPTION_SECURE |
                                       CEF_SCHEME_OPTION_CORS_ENABLED | CEF_SCHEME_OPTION_FETCH_ENABLED);
    }
}

void IRuntime::OnBeforeCommandLineProcessing(const CefString &process_type, CefRefPtr<CefCommandLine> command_line)
{
    command_line->AppendSwitch("use-mock-keychain");
}

void IRuntime::OnContextInitialized()
{
    if (_custom_scheme.has_value())
    {
        CefRegisterSchemeHandlerFactory(_custom_scheme.value().name,
                                        _custom_scheme.value().domain,
                                        new ISchemeHandlerFactory(_custom_scheme.value()));
    }

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

void IRuntime::OnBeforeChildProcessLaunch(CefRefPtr<CefCommandLine> command_line)
{
    if (_custom_scheme.has_value())
    {
        command_line->AppendSwitchWithValue("scheme-name", _custom_scheme.value().name);
    }
}

CefSettings &IRuntime::GetCefSettings()
{
    return _cef_settings;
}

CefRefPtr<IWebView> IRuntime::CreateWebView(std::string url, const WebViewSettings *settings, WebViewHandler handler)
{
    CefBrowserSettings broswer_settings;

    // clang-format off
    broswer_settings.background_color = 0xFF;
    broswer_settings.default_font_size = settings->default_font_size;
    broswer_settings.windowless_frame_rate = settings->windowless_frame_rate;
    broswer_settings.default_fixed_font_size = settings->default_fixed_font_size;
    broswer_settings.webgl = settings->webgl ? STATE_ENABLED : STATE_DISABLED;
    broswer_settings.databases = settings->databases ? STATE_ENABLED : STATE_DISABLED;
    broswer_settings.local_storage = settings->local_storage ? STATE_ENABLED : STATE_DISABLED;
    broswer_settings.javascript = settings->javascript ? STATE_ENABLED : STATE_DISABLED;
    broswer_settings.javascript_access_clipboard = settings->javascript_access_clipboard ? STATE_ENABLED : STATE_DISABLED;
    // clang-format on

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
    else
    {
        window_info.SetAsWindowless(nullptr);
    }

    CefRefPtr<IWebView> webview = new IWebView(_cef_settings, settings, handler);
    if (!CefBrowserHost::CreateBrowser(window_info, webview, url, broswer_settings, nullptr, nullptr))
    {
        return nullptr;
    }

    return webview;
}

int get_result_code()
{
    return CefGetExitCode();
}

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

    return new Runtime{new IRuntime(settings, cef_settings, handler)};
}

bool execute_runtime(void *runtime, int argc, const char **argv)
{
    assert(runtime != nullptr);

    auto rt = static_cast<Runtime *>(runtime);
    auto main_args = get_main_args(argc, argv);
    return CefInitialize(main_args, rt->ref->GetCefSettings(), rt->ref, nullptr);
}

void close_runtime(void *runtime)
{
    assert(runtime != nullptr);

    CefShutdown();

    delete static_cast<Runtime *>(runtime);
}

void *create_webview(void *runtime, const char *url, const WebViewSettings *settings, WebViewHandler handler)
{
    assert(runtime != nullptr);
    assert(settings != nullptr);
    assert(url != nullptr);

    auto webview = static_cast<Runtime *>(runtime)->ref->CreateWebView(std::string(url), settings, handler);
    return new WebView{webview};
}
