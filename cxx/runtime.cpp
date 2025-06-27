//
//  runtime.cpp
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#include "runtime.h"

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

IRuntime::~IRuntime()
{
    this->Close();
}

CefRefPtr<CefBrowserProcessHandler> IRuntime::GetBrowserProcessHandler()
{
    CHECK_REFCOUNTING(nullptr);

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
    CHECK_REFCOUNTING();

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
    CHECK_REFCOUNTING();

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
    CHECK_REFCOUNTING(nullptr);

    CefBrowserSettings broswer_settings;

    // clang-format off
    broswer_settings.default_font_size = settings->default_font_size;
    broswer_settings.default_fixed_font_size = settings->default_fixed_font_size;
    broswer_settings.minimum_font_size = settings->minimum_font_size;
    broswer_settings.minimum_logical_font_size = settings->minimum_logical_font_size;
    broswer_settings.webgl = settings->webgl ? STATE_ENABLED : STATE_DISABLED;
    broswer_settings.databases = settings->databases ? STATE_ENABLED : STATE_DISABLED;
    broswer_settings.local_storage = settings->local_storage ? STATE_ENABLED : STATE_DISABLED;
    broswer_settings.javascript = settings->javascript ? STATE_ENABLED : STATE_DISABLED;
    broswer_settings.javascript_access_clipboard = settings->javascript_access_clipboard ? STATE_ENABLED : STATE_DISABLED;
    broswer_settings.javascript_dom_paste = settings->javascript_dom_paste ? STATE_ENABLED : STATE_DISABLED;
    broswer_settings.javascript_close_windows = settings->javascript_close_windows ? STATE_ENABLED : STATE_DISABLED;
    broswer_settings.background_color = settings->background_color;
    broswer_settings.windowless_frame_rate = settings->windowless_frame_rate;
    // clang-format on

    CefWindowInfo window_info;
    if (_cef_settings.windowless_rendering_enabled)
    {
        window_info.SetAsWindowless(settings->window_handle != nullptr ? (CefWindowHandle)(settings->window_handle)
                                                                       : nullptr);
    }
    else
    {
        if (settings->window_handle != nullptr)
        {
            CefRect rect(0, 0, settings->width, settings->height);
            window_info.SetAsChild((CefWindowHandle)(settings->window_handle), rect);
        }
    }

    CefRefPtr<IWebView> webview = new IWebView(_cef_settings, settings, handler);
    if (!CefBrowserHost::CreateBrowser(window_info, webview, url, broswer_settings, nullptr, nullptr))
    {
        return nullptr;
    }

    return webview;
}

void IRuntime::Close()
{
    CLOSE_RUNNING;
}
