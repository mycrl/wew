//
//  subprocess.h
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#ifndef subprocess_h
#define subprocess_h
#pragma once

#include <optional>
#include <string>

#include "include/cef_app.h"
#include "wew.h"

class MessageSender : public CefV8Handler
{
  public:
    bool Execute(const CefString &name,
                 CefRefPtr<CefV8Value> object,
                 const CefV8ValueList &arguments,
                 CefRefPtr<CefV8Value> &retval,
                 CefString &exception) override;

    void SetBrowser(CefRefPtr<CefBrowser> browser)
    {
        _browser = std::optional(browser);
    }

  private:
    std::optional<CefRefPtr<CefBrowser>> _browser = std::nullopt;

    IMPLEMENT_REFCOUNTING(MessageSender);
};

class MessageReceiver : public CefV8Handler
{
  public:
    bool Execute(const CefString &name,
                 CefRefPtr<CefV8Value> object,
                 const CefV8ValueList &arguments,
                 CefRefPtr<CefV8Value> &retval,
                 CefString &exception) override;

    void Recv(std::string message);

  private:
    std::optional<CefRefPtr<CefV8Context>> _context = std::nullopt;
    std::optional<CefRefPtr<CefV8Value>> _callback = std::nullopt;

    IMPLEMENT_REFCOUNTING(MessageReceiver);
};

class ISubProcess : public CefApp, public CefRenderProcessHandler
{
  public:
    /* CefApp */

    ///
    /// Provides an opportunity to register custom schemes.
    ///
    void OnRegisterCustomSchemes(CefRawPtr<CefSchemeRegistrar> registrar) override;

    ///
    /// Return the handler for functionality specific to the render process.
    ///
    CefRefPtr<CefRenderProcessHandler> GetRenderProcessHandler() override;

    /* CefRenderProcessHandler */

    ///
    /// Called immediately after the V8 context for a frame has been created.
    ///
    void OnContextCreated(CefRefPtr<CefBrowser> browser,
                          CefRefPtr<CefFrame> frame,
                          CefRefPtr<CefV8Context> context) override;

    ///
    /// Called when a new message is received from a different process.
    ///
    /// Return true if the message was handled or false otherwise. It is safe to keep a reference to |message| outside
    /// of this callback.
    ///
    bool OnProcessMessageReceived(CefRefPtr<CefBrowser> browser,
                                  CefRefPtr<CefFrame> frame,
                                  CefProcessId source_process,
                                  CefRefPtr<CefProcessMessage> message) override;

  private:
    CefRefPtr<MessageSender> _sender = new MessageSender();
    CefRefPtr<MessageReceiver> _receiver = new MessageReceiver();

    IMPLEMENT_REFCOUNTING(ISubProcess);
};

#endif /* subprocess_h */
