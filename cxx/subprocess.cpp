//
//  subprocess.cpp
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#ifdef MACOS
#include "include/wrapper/cef_library_loader.h"
#endif

#include "library.h"
#include "subprocess.h"
#include "util.h"

int execute_subprocess(int argc, const char **argv)
{
#ifdef MACOS
    CefScopedLibraryLoader library_loader;
    if (!library_loader.LoadInHelper())
    {
        return -1;
    }
#endif

    auto main_args = get_main_args(argc, argv);
    return CefExecuteProcess(main_args, new ISubProcess, nullptr);
}

CefRefPtr<CefRenderProcessHandler> ISubProcess::GetRenderProcessHandler()
{
    return this;
}

void ISubProcess::OnContextCreated(CefRefPtr<CefBrowser> browser,
                                   CefRefPtr<CefFrame> frame,
                                   CefRefPtr<CefV8Context> context)
{
    _sender->SetBrowser(browser);

    CefRefPtr<CefV8Value> native = CefV8Value::CreateObject(nullptr, nullptr);
    native->SetValue("send", CefV8Value::CreateFunction("send", _sender), V8_PROPERTY_ATTRIBUTE_NONE);
    native->SetValue("recv", CefV8Value::CreateFunction("recv", _receiver), V8_PROPERTY_ATTRIBUTE_NONE);

    CefRefPtr<CefV8Value> global = context->GetGlobal();
    global->SetValue("WebViewMessageChannel", std::move(native), V8_PROPERTY_ATTRIBUTE_NONE);
}

bool ISubProcess::OnProcessMessageReceived(CefRefPtr<CefBrowser> browser,
                                           CefRefPtr<CefFrame> frame,
                                           CefProcessId source_process,
                                           CefRefPtr<CefProcessMessage> message)
{
    auto args = message->GetArgumentList();
    std::string payload = args->GetString(0);
    _receiver->Recv(payload);
    return true;
}

bool MessageSender::Execute(const CefString &name,
                            CefRefPtr<CefV8Value> object,
                            const CefV8ValueList &arguments,
                            CefRefPtr<CefV8Value> &retval,
                            CefString &exception)
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

bool MessageReceiver::Execute(const CefString &name,
                              CefRefPtr<CefV8Value> object,
                              const CefV8ValueList &arguments,
                              CefRefPtr<CefV8Value> &retval,
                              CefString &exception)
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

void MessageReceiver::Recv(std::string message)
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
