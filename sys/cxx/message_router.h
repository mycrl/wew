//
//  message_router.h
//  webview
//
//  Created by Mr.Panda on 2023/7/4.
//

#ifndef LIBWEBVIEW_MESSAGE_ROUTER_H
#define LIBWEBVIEW_MESSAGE_ROUTER_H
#pragma once

#include <functional>
#include <map>
#include <memory>
#include <mutex>
#include <optional>
#include <string>

#include "include/cef_app.h"

class MessageRouter
{
public:
    typedef std::function<void(std::string&)> Handler;

    ~MessageRouter()
    {
        IClose();
    }

    void Send(int source_id, std::string& msg);
    void On(int id, Handler handler);
    void RemoveHandler(int id);

    void IClose();

private:
    std::mutex _mutex;
    std::map<int, Handler> _map;
    bool _is_closed = false;
};

class MessageRouterMaster
{
public:
    MessageRouterMaster(int id, std::shared_ptr<MessageRouter> router);
    ~MessageRouterMaster()
    {
        IClose();
    }

    void IClose();
    void SetBrowser(CefRefPtr<CefBrowser> browser);
    void OnMessage(CefRefPtr<CefProcessMessage> msg);

private:
    void _Send(std::string& payload);

    std::optional<CefRefPtr<CefBrowser>> _browser = std::nullopt;

    std::shared_ptr<MessageRouter> _router;
    bool _is_closed = false;
    int _id;
};

class MessageRouterHost
{
public:
    typedef std::function<void(std::string&)> Handler;
    ~MessageRouterHost()
    {
        IClose();
    }

    void IClose();
    void SetBrowser(CefRefPtr<CefBrowser> browser);
    void Broadcast(std::string& payload);
    void On(Handler handler);
    void OnMessage(CefRefPtr<CefProcessMessage> msg);

private:
    std::optional<CefRefPtr<CefBrowser>> _browser = std::nullopt;
    std::optional<Handler> _handler = std::nullopt;

    bool _is_closed = false;
};

#endif  // LIBWEBVIEW_MESSAGE_ROUTER_H