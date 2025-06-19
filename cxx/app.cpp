//
//  app.cpp
//  webview
//
//  Created by Mr.Panda on 2023/4/26.
//

#include "app.h"
#include "include/wrapper/cef_helpers.h"
#include "scheme_handler.h"

IApp::IApp(const AppOptions* settings, AppObserver observer, void* ctx)
: _observer(observer), _ctx(ctx)
{
    assert(settings != nullptr);
    
    cef_settings.no_sandbox = true;
    cef_settings.command_line_args_disabled = true;
    cef_settings.windowless_rendering_enabled = settings->windowless_rendering_enabled;
    cef_settings.multi_threaded_message_loop = settings->multi_threaded_message_loop;
    cef_settings.external_message_pump = settings->external_message_pump;
    cef_settings.background_color = 0xFF;
    
    CefString(&cef_settings.locale).FromString("en-US");
    
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
    
    if (settings->scheme_dir_path != nullptr)
    {
        _scheme_dir_path = std::string(settings->scheme_dir_path);
    }
}

CefRefPtr<CefBrowserProcessHandler> IApp::GetBrowserProcessHandler()
{
    return this;
}

void IApp::OnContextInitialized()
{
    CEF_REQUIRE_UI_THREAD();
    
    if (_scheme_dir_path.has_value())
    {
        RegisterSchemeHandlerFactory(_scheme_dir_path.value());
    }
    
    _observer.on_context_initialized(_ctx);
}

CefRefPtr<CefClient> IApp::GetDefaultClient()
{
    return nullptr;
}

void IApp::OnScheduleMessagePumpWork(int64_t delay_ms)
{
    _observer.on_schedule_message_pump_work(delay_ms, _ctx);
}

void IApp::OnBeforeCommandLineProcessing(const CefString& process_type,
                                         CefRefPtr<CefCommandLine> command_line) 
{
    command_line->AppendSwitch("use-mock-keychain");
}

CefRefPtr<IPage> IApp::CreatePage(std::string url,
                                  const PageOptions* settings,
                                  PageObserver observer,
                                  void* ctx)
{
    assert(settings != nullptr);
    
    CefBrowserSettings broswer_settings;
    broswer_settings.webgl = cef_state_t::STATE_DISABLED;
    broswer_settings.databases = cef_state_t::STATE_DISABLED;
    broswer_settings.background_color = 0xFF;
    
    broswer_settings.default_font_size = settings->default_font_size;
    broswer_settings.windowless_frame_rate = settings->windowless_frame_rate;
    broswer_settings.default_fixed_font_size = settings->default_fixed_font_size;
    broswer_settings.local_storage = settings->local_storage ? STATE_ENABLED : STATE_DISABLED;
    broswer_settings.javascript = settings->javascript ? STATE_ENABLED : STATE_DISABLED;
    broswer_settings.javascript_access_clipboard = settings->javascript_access_clipboard ? STATE_ENABLED : STATE_DISABLED;
    
    CefWindowInfo window_info;
    if (settings->window_handle != nullptr)
    {
        if (cef_settings.windowless_rendering_enabled)
        {
            window_info.SetAsWindowless((CefWindowHandle)(settings->window_handle));
        }
        else
        {
            window_info.SetAsChild((CefWindowHandle)(settings->window_handle),
                                   CefRect(0, 0, settings->width, settings->height));
        }
    }
    
    CefRefPtr<IPage> page = new IPage(cef_settings, settings, observer, ctx);
    CefBrowserHost::CreateBrowser(window_info, page, url, broswer_settings, nullptr, nullptr);
    return page;
}

void IApp::OnRegisterCustomSchemes(CefRawPtr<CefSchemeRegistrar> registrar)
{
    registrar->AddCustomScheme(WEBVIEW_SCHEME_NAME,
                               CEF_SCHEME_OPTION_STANDARD | CEF_SCHEME_OPTION_SECURE |
                               CEF_SCHEME_OPTION_CORS_ENABLED | CEF_SCHEME_OPTION_FETCH_ENABLED);
}

CefRefPtr<CefRenderProcessHandler> IRenderApp::GetRenderProcessHandler()
{
    return this;
}

void IRenderApp::OnRegisterCustomSchemes(CefRawPtr<CefSchemeRegistrar> registrar)
{
    registrar->AddCustomScheme(WEBVIEW_SCHEME_NAME,
                               CEF_SCHEME_OPTION_STANDARD | CEF_SCHEME_OPTION_SECURE |
                               CEF_SCHEME_OPTION_CORS_ENABLED | CEF_SCHEME_OPTION_FETCH_ENABLED);
}

void IRenderApp::OnContextCreated(CefRefPtr<CefBrowser> browser,
                                  CefRefPtr<CefFrame> frame,
                                  CefRefPtr<CefV8Context> context)
{
    _send_func->SetBrowser(browser);
    
    CefRefPtr<CefV8Value> native = CefV8Value::CreateObject(nullptr, nullptr);
    native->SetValue("send", CefV8Value::CreateFunction("send", _send_func), V8_PROPERTY_ATTRIBUTE_NONE);
    native->SetValue("on", CefV8Value::CreateFunction("on", _on_func), V8_PROPERTY_ATTRIBUTE_NONE);
    
    CefRefPtr<CefV8Value> global = context->GetGlobal();
    global->SetValue("MessageTransport", std::move(native), V8_PROPERTY_ATTRIBUTE_NONE);
}

bool IRenderApp::OnProcessMessageReceived(CefRefPtr<CefBrowser> browser,
                                          CefRefPtr<CefFrame> frame,
                                          CefProcessId source_process,
                                          CefRefPtr<CefProcessMessage> message)
{
    auto args = message->GetArgumentList();
    std::string payload = args->GetString(0);
    _on_func->Call(payload);
    return true;
}

bool MessageSendFunction::Execute(const CefString& name,
                                  CefRefPtr<CefV8Value> object,
                                  const CefV8ValueList& arguments,
                                  CefRefPtr<CefV8Value>& retval,
                                  CefString& exception)
{
    if (!_browser.has_value())
    {
        return false;
    }
    
    if (arguments.size() != 1)
    {
        return false;
    }
    
    if (!arguments[0]->IsString())
    {
        return false;
    }
    
    CefRefPtr<CefV8Context> context = CefV8Context::GetCurrentContext();
    std::string message = arguments[0]->GetStringValue();
    
    auto msg = CefProcessMessage::Create("MESSAGE_TRANSPORT");
    CefRefPtr<CefListValue> args = msg->GetArgumentList();
    args->SetSize(1);
    args->SetString(0, message);
    
    _browser.value()->GetMainFrame()->SendProcessMessage(PID_BROWSER, msg);
    retval = CefV8Value::CreateUndefined();
    return true;
}

bool MessageOnFunction::Execute(const CefString& name,
                                CefRefPtr<CefV8Value> object,
                                const CefV8ValueList& arguments,
                                CefRefPtr<CefV8Value>& retval,
                                CefString& exception)
{
    if (arguments.size() != 1)
    {
        return false;
    }
    
    if (!arguments[0]->IsFunction())
    {
        return false;
    }
    
    _context = std::optional(CefV8Context::GetCurrentContext());
    _callback = std::optional(arguments[0]);
    retval = CefV8Value::CreateUndefined();
    return true;
}

void MessageOnFunction::Call(std::string message)
{
    if (!_context.has_value())
    {
        return;
    }
    
    if (!_callback.has_value())
    {
        return;
    }
    
    _context.value()->Enter();
    CefV8ValueList arguments;
    arguments.push_back(CefV8Value::CreateString(message));
    _callback.value()->ExecuteFunction(nullptr, arguments);
    _context.value()->Exit();
}
