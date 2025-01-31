//
//  bridge.cpp
//  webview
//
//  Created by Mr.Panda on 2023/5/5.
//

#include "bridge.h"

using namespace std::placeholders;

/* ================= MessageTransPort =======================*/

void MessageTransPort::Call(const std::string& req, Handler handler)
{
    if (_is_closed)
    {
        handler(CLOSED_ERR, true);
        return;
    }

    if (!_browser.has_value())
    {
        handler(NOT_HANDLER_ERR, true);
        return;
    }

    auto seq = _GetSeqNumber();
    auto msg = CefProcessMessage::Create("__inner_call_request");
    CefRefPtr<CefListValue> args = msg->GetArgumentList();
    args->SetSize(2);
    args->SetString(0, req);
    args->SetInt(1, seq);

    auto pid = _is_master ? PID_RENDERER : PID_BROWSER;
    _browser.value()->GetMainFrame()->SendProcessMessage(pid, msg);

    _mutex.lock();
    _call_table.insert({ seq, handler });
    _mutex.unlock();
}

bool MessageTransPort::OnMessage(CefRefPtr<CefProcessMessage> msg)
{
    if (_is_closed)
    {
        return false;
    }

    std::string kind_name = msg->GetName();
    CefRefPtr<CefListValue> args = msg->GetArgumentList();
    int seq_id = args->GetInt(args->GetSize() - 1);

    if (kind_name == "__inner_call_request")
    {
        _HandleCallRequest(args, seq_id);
        return true;
    }

    if (kind_name == "__inner_call_response")
    {
        _HandleCallResponse(args, seq_id);
        return true;
    }

    return false;
}

void MessageTransPort::On(OnHandler handler)
{
    if (!_is_closed)
    {
        _on_handler = handler;
    }
}

void MessageTransPort::IClose()
{
    _is_closed = true;
    _browser = std::nullopt;
    _on_handler = std::nullopt;
}

int MessageTransPort::_GetSeqNumber()
{
    std::lock_guard<std::mutex> lock(_mutex);

    int number = _seq;
    _seq = _seq == INT_MAX ? 0 : _seq + 1;
    return number;
}

void MessageTransPort::_HandleCallRequest(CefRefPtr<CefListValue> args, int seq_id)
{
    if (_is_closed)
    {
        return;
    }

    if (!_on_handler.has_value())
    {
        _OnHandleCallback(NOT_HANDLER_ERR, true, seq_id);
        return;
    }

    std::string req = args->GetString(0);
    _on_handler.value()(
        req, [=](std::string& res, bool is_err) { _OnHandleCallback(res, is_err, seq_id); });
}

void MessageTransPort::_HandleCallResponse(CefRefPtr<CefListValue> args, int seq_id)
{
    if (_is_closed)
    {
        return;
    }

    bool is_err = args->GetBool(0);
    std::string res = args->GetString(1);
    auto iter = _call_table.find(seq_id);
    if (iter == _call_table.end())
    {
        return;
    }

    std::pair<int, Handler> kv = *iter;
    kv.second(res, is_err);
    _call_table.erase(iter);
}

void MessageTransPort::_OnHandleCallback(std::string& res, bool is_err, int seq_id)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    auto msg = CefProcessMessage::Create("__inner_call_response");
    CefRefPtr<CefListValue> args = msg->GetArgumentList();
    args->SetSize(3);
    args->SetBool(0, is_err);
    args->SetString(1, res);
    args->SetInt(2, seq_id);

    auto pid = _is_master ? PID_RENDERER : PID_BROWSER;
    _browser.value()->GetMainFrame()->SendProcessMessage(pid, msg);
}

/* ================= IpcSendProcesser =======================*/

bool IpcSendProcesser::Execute(const CefString& name_,
                               CefRefPtr<CefV8Value> object,
                               const CefV8ValueList& arguments,
                               CefRefPtr<CefV8Value>& retval,
                               CefString& exception)
{
    if (arguments.size() != 1)
    {
        return false;
    }

    if (!arguments[0]->IsString())
    {
        return false;
    }

    std::string payload = arguments[0]->GetStringValue();
    _router_host->Broadcast(payload);

    retval = CefV8Value::CreateUndefined();
    return true;
}

/* ================= IpcOnProcesser =======================*/

bool IpcOnProcesser::Execute(const CefString& name_,
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

    CefRefPtr<CefV8Value> callback = arguments[0];
    CefRefPtr<CefV8Context> context = CefV8Context::GetCurrentContext();
    _router_host->On([=](std::string& payload) { _OnCallback(context, callback, payload); });

    retval = CefV8Value::CreateUndefined();
    return true;
}

void IpcOnProcesser::_OnCallback(CefRefPtr<CefV8Context> context,
                                 CefRefPtr<CefV8Value> callback,
                                 std::string& payload)
{
    context->Enter();
    CefV8ValueList arguments;
    arguments.push_back(CefV8Value::CreateString(payload));
    callback->ExecuteFunction(nullptr, arguments);
    context->Exit();
}

/* ================= BridgeCallbacker =======================*/

bool BridgeCallbacker::Execute(const CefString& name_,
                               CefRefPtr<CefV8Value> object,
                               const CefV8ValueList& arguments,
                               CefRefPtr<CefV8Value>& retval,
                               CefString& exception)
{
    if (arguments.size() != 2)
    {
        return false;
    }

    if (!arguments[0]->IsBool())
    {
        return false;
    }

    if (!arguments[1]->IsString())
    {
        return false;
    }

    bool is_err = arguments[0]->GetBoolValue();
    std::string res = arguments[1]->GetStringValue();

    _handler(res, is_err);
    retval = CefV8Value::CreateUndefined();
    return true;
}

/* ================= BridgeOnProcesser =======================*/

bool BridgeOnProcesser::Execute(const CefString& name_,
                                CefRefPtr<CefV8Value> object,
                                const CefV8ValueList& arguments,
                                CefRefPtr<CefV8Value>& retval,
                                CefString& exception)
{
    if (arguments.size() < 1)
    {
        return false;
    }

    if (!arguments[0]->IsFunction())
    {
        return false;
    }

    CefRefPtr<CefV8Context> context = CefV8Context::GetCurrentContext();
    CefRefPtr<CefV8Value> callback = arguments[0];

    _transport->On([=](std::string& req, MessageTransPort::Handler handler) {
        _HandleOnCallback(context, callback, req, handler);
                   });

    retval = CefV8Value::CreateUndefined();
    return true;
}

void BridgeOnProcesser::_HandleOnCallback(CefRefPtr<CefV8Context> context,
                                          CefRefPtr<CefV8Value> callback,
                                          std::string& req,
                                          MessageTransPort::Handler handler)
{
    context->Enter();
    CefV8ValueList arguments;
    arguments.push_back(CefV8Value::CreateString(req));
    arguments.push_back(CefV8Value::CreateFunction("callback", new BridgeCallbacker(handler)));
    callback->ExecuteFunction(nullptr, arguments);
    context->Exit();
}

/* ================= BridgeCallProcesser =======================*/

bool BridgeCallProcesser::Execute(const CefString& name,
                                  CefRefPtr<CefV8Value> object,
                                  const CefV8ValueList& arguments,
                                  CefRefPtr<CefV8Value>& retval,
                                  CefString& exception)
{
    if (arguments.size() != 2)
    {
        return false;
    }

    if (!arguments[0]->IsString())
    {
        return false;
    }

    if (!arguments[1]->IsFunction())
    {
        return false;
    }

    CefRefPtr<CefV8Context> context = CefV8Context::GetCurrentContext();
    std::string req = arguments[0]->GetStringValue();
    CefRefPtr<CefV8Value> callback = arguments[1];

    _transport->Call(
        req, [=](std::string& res, bool is_err) { _HandleCallback(callback, context, res, is_err); });

    retval = CefV8Value::CreateUndefined();
    return true;
}

void BridgeCallProcesser::_HandleCallback(CefRefPtr<CefV8Value> callback,
                                          CefRefPtr<CefV8Context> context,
                                          std::string& res,
                                          bool is_err)
{
    context->Enter();
    CefV8ValueList arguments;

    auto nul = CefV8Value::CreateNull();
    auto ret = CefV8Value::CreateString(res);

    arguments.push_back(is_err ? ret : nul);
    arguments.push_back(is_err ? nul : ret);
    callback->ExecuteFunction(nullptr, arguments);
    context->Exit();
}

/* ================= IBridgeHost =======================*/

#define CREATE_FUNC(name, func) \
  name, CefV8Value::CreateFunction(name, func), V8_PROPERTY_ATTRIBUTE_NONE

#define CREATE_PROPERTY(name, value) name, std::move(value), V8_PROPERTY_ATTRIBUTE_NONE

void IBridgeHost::OnContextCreated(CefRefPtr<CefBrowser> browser,
                                   CefRefPtr<CefFrame> frame,
                                   CefRefPtr<CefV8Context> context)
{
    CefRefPtr<CefV8Value> bridge = CefV8Value::CreateObject(nullptr, nullptr);
    bridge->SetValue(CREATE_FUNC("call", _bridge_call));
    bridge->SetValue(CREATE_FUNC("on", _bridge_on));

    CefRefPtr<CefV8Value> ipc = CefV8Value::CreateObject(nullptr, nullptr);
    ipc->SetValue(CREATE_FUNC("send", _ipc_send));
    ipc->SetValue(CREATE_FUNC("on", _ipc_on));

    CefRefPtr<CefV8Value> native = CefV8Value::CreateObject(nullptr, nullptr);
    native->SetValue(CREATE_PROPERTY("bridge", bridge));
    native->SetValue(CREATE_PROPERTY("ipc", ipc));

    CefRefPtr<CefV8Value> global = context->GetGlobal();
    global->SetValue(CREATE_PROPERTY("native", native));

    _transport->SetBrowser(browser);
    _router_host->SetBrowser(browser);
}

bool IBridgeHost::OnProcessMessageReceived(CefRefPtr<CefBrowser> browser,
                                           CefRefPtr<CefFrame> frame,
                                           CefProcessId source_process,
                                           CefRefPtr<CefProcessMessage> message)
{
    if (!_transport->OnMessage(message))
    {
        _router_host->OnMessage(message);
    }

    return true;
}

/* ================= IBridgeMaster =======================*/

void IBridgeMaster::BridgeMasterOnMessage(CefRefPtr<CefProcessMessage> message)
{
    if (_is_closed)
    {
        return;
    }

    if (!_router_master.has_value())
    {
        return;
    }

    if (!_transport->OnMessage(message))
    {
        _router_master.value()->OnMessage(message);
    }
}

void IBridgeMaster::SetBrowser(CefRefPtr<CefBrowser> browser)
{
    if (_is_closed)
    {
        return;
    }

    _browser = browser;
    _transport->SetBrowser(browser);
    _transport->On(
        [&](std::string& req, MessageTransPort::Handler handler) { _HandleOn(req, handler); });

    auto id = _browser.value()->GetIdentifier();
    _router_master = std::make_shared<MessageRouterMaster>(id, _router);
    _router_master.value()->SetBrowser(browser);
}

void IBridgeMaster::BridgeCall(char* req, BridgeCallCallback callback, void* ctx)
{
    assert(req);
    assert(callback);

    if (_is_closed)
    {
        callback(nullptr, ctx);
        return;
    }

    if (!_browser.has_value())
    {
        callback(nullptr, ctx);
        return;
    }

    _transport->Call(std::string(req), [=](std::string& res, bool is_err) {
        callback(is_err ? nullptr : res.c_str(), ctx);
                     });
}

void IBridgeMaster::BridgeSetOnCallback(BridgeOnHandler handler, void* ctx)
{
    assert(handler);

    if (_is_closed)
    {
        return;
    }

    _handler = handler;
    _ctx = ctx;
}

void IBridgeMaster::BridgeRemoveOnCallback()
{
    _handler = std::nullopt;
    _ctx = std::nullopt;
}

void IBridgeMaster::_HandleOn(std::string& req, MessageTransPort::Handler handler)
{
    if (_is_closed)
    {
        return;
    }

    if (_ctx.has_value() && _handler.has_value())
    {
        _handler.value()(req.c_str(), _ctx.value(), new Context(handler),
                         bridge_master_handler_callback);
    }
    else
    {
        auto res = std::string("runtime not load!");
        handler(res, true);
    }
}

void IBridgeMaster::IClose()
{
    if (_router_master.has_value())
    {
        _router_master.value()->IClose();
    }

    _router->IClose();
    _transport->IClose();
    BridgeRemoveOnCallback();

    _router_master = std::nullopt;
    _browser = std::nullopt;
    _is_closed = true;
}

void bridge_master_handler_callback(void* ctx, Result ret)
{
    assert(ctx);

    IBridgeMaster::Context* ictx = (IBridgeMaster::Context*)ctx;
    std::string res = ret.success ? std::string(ret.success) : std::string(ret.failure);
    ictx->handler(res, ret.failure != nullptr);
    delete ictx;
}
