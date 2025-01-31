//
//  bridge.h
//  webview
//
//  Created by Mr.Panda on 2023/5/5.
//

#ifndef LIBWEBVIEW_BRIDGE_H
#define LIBWEBVIEW_BRIDGE_H
#pragma once

#include <functional>
#include <map>
#include <memory>
#include <mutex>
#include <optional>

#include "include/cef_app.h"
#include "message_router.h"
#include "webview.h"

/* =================== MessageTransPort ====================== */

void bridge_master_handler_callback(void* ctx, Result ret);

class MessageTransPort
{
public:
    std::string CLOSED_ERR = std::string("is closed!");
    std::string NOT_HANDLER_ERR = std::string("not set handler!");

    typedef std::function<void(std::string&, bool)> Handler;
    typedef std::function<void(std::string&, Handler)> OnHandler;

    MessageTransPort(bool is_master) : _is_master(is_master)
    {
    }

    ~MessageTransPort()
    {
        IClose();
    }

    void SetBrowser(CefRefPtr<CefBrowser> browser)
    {
        _browser = browser;
    }

    void Call(const std::string& req, Handler handler);
    void On(OnHandler handler);

    bool OnMessage(CefRefPtr<CefProcessMessage> msg);
    void IClose();

private:
    void _HandleCallRequest(CefRefPtr<CefListValue> args, int seq_id);
    void _HandleCallResponse(CefRefPtr<CefListValue> args, int seq_id);
    void _OnHandleCallback(std::string& res, bool is_err, int seq_id);
    int _GetSeqNumber();

    std::optional<CefRefPtr<CefBrowser>> _browser = std::nullopt;
    std::optional<OnHandler> _on_handler = std::nullopt;

    std::mutex _mutex;
    std::map<int, Handler> _call_table;
    bool _is_closed = false;
    bool _is_master = false;
    int _seq = 0;
};

/* =================== BridgeCallbacker ====================== */

class BridgeCallbacker : public CefV8Handler
{
public:
    BridgeCallbacker(MessageTransPort::Handler handler) : _handler(handler)
    {
    }

    /* CefV8Handler */

    bool Execute(const CefString& name_,
                 CefRefPtr<CefV8Value> object,
                 const CefV8ValueList& arguments,
                 CefRefPtr<CefV8Value>& retval,
                 CefString& exception);

private:
    MessageTransPort::Handler _handler;

    IMPLEMENT_REFCOUNTING(BridgeCallbacker);
};

/* =================== IpcSendProcesser ====================== */

class IpcSendProcesser : public CefV8Handler
{
public:
    IpcSendProcesser(std::shared_ptr<MessageRouterHost> router_host) : _router_host(router_host)
    {
    }

    /* CefV8Handler */

    bool Execute(const CefString& name_,
                 CefRefPtr<CefV8Value> object,
                 const CefV8ValueList& arguments,
                 CefRefPtr<CefV8Value>& retval,
                 CefString& exception);

private:
    std::shared_ptr<MessageRouterHost> _router_host;

    IMPLEMENT_REFCOUNTING(IpcSendProcesser);
};

/* =================== IpcOnProcesser ====================== */

class IpcOnProcesser : public CefV8Handler
{
public:
    IpcOnProcesser(std::shared_ptr<MessageRouterHost> router_host) : _router_host(router_host)
    {
    }

    /* CefV8Handler */

    bool Execute(const CefString& name_,
                 CefRefPtr<CefV8Value> object,
                 const CefV8ValueList& arguments,
                 CefRefPtr<CefV8Value>& retval,
                 CefString& exception);

private:
    void _OnCallback(CefRefPtr<CefV8Context> context,
                     CefRefPtr<CefV8Value> callback,
                     std::string& payload);

    std::shared_ptr<MessageRouterHost> _router_host;

    IMPLEMENT_REFCOUNTING(IpcOnProcesser);
};

/* =================== BridgeOnProcesser ====================== */

class BridgeOnProcesser : public CefV8Handler
{
public:
    BridgeOnProcesser(std::shared_ptr<MessageTransPort> transport) : _transport(transport)
    {
    }

    /* CefV8Handler */

    bool Execute(const CefString& name_,
                 CefRefPtr<CefV8Value> object,
                 const CefV8ValueList& arguments,
                 CefRefPtr<CefV8Value>& retval,
                 CefString& exception);

    void _HandleOnCallback(CefRefPtr<CefV8Context> context,
                           CefRefPtr<CefV8Value> callback,
                           std::string& req,
                           MessageTransPort::Handler handler);

private:
    std::shared_ptr<MessageTransPort> _transport;

    IMPLEMENT_REFCOUNTING(BridgeOnProcesser);
};

/* =================== BridgeCallProcesser ====================== */

class BridgeCallProcesser : public CefV8Handler
{
public:
    BridgeCallProcesser(std::shared_ptr<MessageTransPort> transport) : _transport(transport)
    {
    }

    /* CefV8Handler */

    bool Execute(const CefString& name,
                 CefRefPtr<CefV8Value> object,
                 const CefV8ValueList& arguments,
                 CefRefPtr<CefV8Value>& retval,
                 CefString& exception);

private:
    void _HandleCallback(CefRefPtr<CefV8Value> callback,
                         CefRefPtr<CefV8Context> context,
                         std::string& res,
                         bool is_err);

    std::shared_ptr<MessageTransPort> _transport;

    IMPLEMENT_REFCOUNTING(BridgeCallProcesser);
};

/* =================== IBridgeHost ====================== */

class IBridgeHost : public CefRenderProcessHandler
{
public:
    ~IBridgeHost()
    {
        _transport->IClose();
        _router_host->IClose();
    }

    /* CefRenderProcessHandler */

    void OnContextCreated(CefRefPtr<CefBrowser> browser,
                          CefRefPtr<CefFrame> frame,
                          CefRefPtr<CefV8Context> context);
    bool OnProcessMessageReceived(CefRefPtr<CefBrowser> browser,
                                  CefRefPtr<CefFrame> frame,
                                  CefProcessId source_process,
                                  CefRefPtr<CefProcessMessage> message);

private:
    std::shared_ptr<MessageTransPort> _transport = std::make_shared<MessageTransPort>(false);
    std::shared_ptr<MessageRouterHost> _router_host = std::make_shared<MessageRouterHost>();
    CefRefPtr<BridgeCallProcesser> _bridge_call = new BridgeCallProcesser(_transport);
    CefRefPtr<BridgeOnProcesser> _bridge_on = new BridgeOnProcesser(_transport);
    CefRefPtr<IpcSendProcesser> _ipc_send = new IpcSendProcesser(_router_host);
    CefRefPtr<IpcOnProcesser> _ipc_on = new IpcOnProcesser(_router_host);
};

/* =================== IBridgeMaster ====================== */

class IBridgeMaster
{
public:
    class Context
    {
    public:
        Context(MessageTransPort::Handler handler_) : handler(handler_)
        {
        }

        MessageTransPort::Handler handler;
    };

    IBridgeMaster(std::shared_ptr<MessageRouter> router) : _router(router)
    {
    }

    ~IBridgeMaster()
    {
        IClose();
    }

    void SetBrowser(CefRefPtr<CefBrowser> browser);
    void BridgeMasterOnMessage(CefRefPtr<CefProcessMessage> message);
    void BridgeCall(char* req, BridgeCallCallback callback, void* ctx);

    void BridgeSetOnCallback(BridgeOnHandler handler, void* ctx);
    void BridgeRemoveOnCallback();
    void IClose();

private:
    void _HandleOn(std::string& req, MessageTransPort::Handler handler);

    std::optional<std::shared_ptr<MessageRouterMaster>> _router_master = std::nullopt;
    std::optional<CefRefPtr<CefBrowser>> _browser = std::nullopt;
    std::optional<BridgeOnHandler> _handler = std::nullopt;
    std::optional<void*> _ctx = std::nullopt;

    std::shared_ptr<MessageRouter> _router;
    std::shared_ptr<MessageTransPort> _transport = std::make_shared<MessageTransPort>(true);
    bool _is_closed = false;
};

#endif  // LIBWEBVIEW_BRIDGE_H
