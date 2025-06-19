//
//  app.h
//  webview
//
//  Created by Mr.Panda on 2023/4/26.
//

#ifndef LIBWEBVIEW_APP_H
#define LIBWEBVIEW_APP_H
#pragma once

#include "page.h"
#include "include/cef_app.h"
#include "webview.h"

class IApp : public CefApp, public CefBrowserProcessHandler
{
public:
    IApp(const AppOptions* settings, AppObserver observer, void* ctx);
    ~IApp()
    {
    }
    
    /* CefApp */
    
    void OnRegisterCustomSchemes(CefRawPtr<CefSchemeRegistrar> registrar) override;
    CefRefPtr<CefBrowserProcessHandler> GetBrowserProcessHandler() override;
    void OnBeforeCommandLineProcessing(const CefString& process_type,
                                       CefRefPtr<CefCommandLine> command_line) override;
    
    /* CefBrowserProcessHandler */
    
    void OnContextInitialized() override;
    CefRefPtr<CefClient> GetDefaultClient() override;
    void OnScheduleMessagePumpWork(int64_t delay_ms) override;
    
    CefRefPtr<IPage> CreatePage(std::string url,
                                const PageOptions* settings,
                                PageObserver observer,
                                void* ctx);
    
    CefSettings cef_settings;
private:
    std::optional<std::string> _scheme_dir_path = std::nullopt;
    AppObserver _observer;
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
                 CefString& exception) override;
    
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
                 CefString& exception) override;
    
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
    CefRefPtr<CefRenderProcessHandler> GetRenderProcessHandler() override;
    
    /* CefRenderProcessHandler */
    
    void OnContextCreated(CefRefPtr<CefBrowser> browser,
                          CefRefPtr<CefFrame> frame,
                          CefRefPtr<CefV8Context> context) override;
    bool OnProcessMessageReceived(CefRefPtr<CefBrowser> browser,
                                  CefRefPtr<CefFrame> frame,
                                  CefProcessId source_process,
                                  CefRefPtr<CefProcessMessage> message) override;
    
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
