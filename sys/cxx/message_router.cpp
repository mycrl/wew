//
//  message_router.cpp
//  webview
//
//  Created by Mr.Panda on 2023/7/4.
//
//
//  message_router.h
//  webview
//
//  Created by Mr.Panda on 2023/7/4.
//

#include "message_router.h"

void MessageRouter::Send(int source_id, std::string& msg)
{
    std::lock_guard<std::mutex> guard(_mutex);
    for (const auto& [key, value] : _map)
    {
        if (key != source_id)
        {
            value(msg);
            break;
        }
    }
}

void MessageRouter::On(int id, Handler handler)
{
    if (_is_closed)
    {
        return;
    }

    std::lock_guard<std::mutex> guard(_mutex);
    _map.insert({ id, handler });
}

void MessageRouter::RemoveHandler(int id)
{
    std::lock_guard<std::mutex> guard(_mutex);
    _map.erase(id);
}

void MessageRouter::IClose()
{
    _is_closed = true;
}

MessageRouterMaster::MessageRouterMaster(int id, std::shared_ptr<MessageRouter> router)
    : _id(id), _router(router)
{
    router->On(id, [=](std::string& payload) { _Send(payload); });
}

void MessageRouterMaster::IClose()
{
    _router->RemoveHandler(_id);
    _browser = std::nullopt;
    _is_closed = true;
}

void MessageRouterMaster::SetBrowser(CefRefPtr<CefBrowser> browser)
{
    if (_is_closed)
    {
        return;
    }

    _browser = browser;
}

void MessageRouterMaster::OnMessage(CefRefPtr<CefProcessMessage> msg)
{
    if (_is_closed)
    {
        return;
    }

    if (msg->GetName() != "__innerMessageRouter")
    {
        return;
    }

    auto args = msg->GetArgumentList();
    std::string payload = args->GetString(0);
    _router->Send(_id, payload);
}

void MessageRouterMaster::_Send(std::string& payload)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    auto msg = CefProcessMessage::Create("__innerMessageRouter");
    CefRefPtr<CefListValue> args = msg->GetArgumentList();
    args->SetSize(1);
    args->SetString(0, payload);
    _browser.value()->GetMainFrame()->SendProcessMessage(PID_RENDERER, msg);
}

void MessageRouterHost::SetBrowser(CefRefPtr<CefBrowser> browser)
{
    if (_is_closed)
    {
        return;
    }

    _browser = browser;
}

void MessageRouterHost::Broadcast(std::string& payload)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    auto msg = CefProcessMessage::Create("__innerMessageRouter");
    CefRefPtr<CefListValue> args = msg->GetArgumentList();
    args->SetSize(1);
    args->SetString(0, payload);
    _browser.value()->GetMainFrame()->SendProcessMessage(PID_BROWSER, msg);
}

void MessageRouterHost::On(Handler handler)
{
    if (_is_closed)
    {
        return;
    }

    _handler = handler;
}

void MessageRouterHost::OnMessage(CefRefPtr<CefProcessMessage> msg)
{
    if (_is_closed)
    {
        return;
    }

    if (msg->GetName() != "__innerMessageRouter")
    {
        return;
    }

    if (!_handler.has_value())
    {
        return;
    }

    auto args = msg->GetArgumentList();
    std::string payload = args->GetString(0);
    _handler.value()(payload);
}

void MessageRouterHost::IClose()
{
    _browser = std::nullopt;
    _handler = std::nullopt;
    _is_closed = true;
}