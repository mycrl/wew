//
//  app.cpp
//  webview
//
//  Created by Mr.Panda on 2023/4/26.
//

#include "app.h"

#include "include/wrapper/cef_helpers.h"
#include "scheme_handler.h"

IApp::IApp(const WebviewOptions* settings, CreateWebviewCallback callback, void* ctx)
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
                                        const PageOptions* settings_ptr,
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

    if (settings.window_handle)
    {
        if (settings.is_offscreen)
        {
            window_info.SetAsWindowless((CefWindowHandle)(settings.window_handle));
        }
        else
        {
            window_info.SetAsChild((CefWindowHandle)(settings.window_handle),
                                   CefRect(0, 0, settings.width, settings.height));
        }
    }

    CefRefPtr<IBrowser> browser = new IBrowser(settings, observer, ctx);
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

void IRenderApp::OnContextCreated(CefRefPtr<CefBrowser> browser,
                                  CefRefPtr<CefFrame> frame,
                                  CefRefPtr<CefV8Context> context)
{
    _send_func->SetBrowser(browser);

    CefRefPtr<CefV8Value> native = CefV8Value::CreateObject(nullptr, nullptr);
    native->SetValue("send", CefV8Value::CreateFunction("send", _send_func), V8_PROPERTY_ATTRIBUTE_NONE);
    native->SetValue("on", CefV8Value::CreateFunction("on", _on_func), V8_PROPERTY_ATTRIBUTE_NONE);

    CefRefPtr<CefV8Value> global = context->GetGlobal();
    global->SetValue("message", std::move(native), V8_PROPERTY_ATTRIBUTE_NONE);
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

    auto msg = CefProcessMessage::Create("");
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
