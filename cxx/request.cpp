//
//  request.cpp
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#include "request.h"

// clang-format off
IResourceHandler::IResourceHandler(const RequestHandlerFactory *factory, RequestHandler *handler)
    : _handler(handler)
    , _factory(factory)
{
    assert(factory != nullptr);
    assert(handler != nullptr);
}
// clang-format on

IResourceHandler::~IResourceHandler()
{
    _handler->destroy(_handler->context);
    _factory->destroy_request_handler(_handler);
}

bool IResourceHandler::Open(CefRefPtr<CefRequest> request, bool &handle_request, CefRefPtr<CefCallback> callback)
{
    bool result = _handler->open(_handler->context);
    handle_request = result;
    return result;
}

void IResourceHandler::GetResponseHeaders(CefRefPtr<CefResponse> response,
                                          int64_t &response_length,
                                          CefString &redirectUrl)
{
    Response res = {.status_code = 0, .content_length = 0, .mime_type = new char[255]};

    _handler->get_response(&res, _handler->context);

    response->SetMimeType(std::string(res.mime_type));
    response->SetStatus(res.status_code);
    response_length = res.content_length;

    delete[] res.mime_type;
}

bool IResourceHandler::Skip(int64_t bytes_to_skip, int64_t &bytes_skipped, CefRefPtr<CefResourceSkipCallback> callback)
{
    int cursor = 0;
    bool result = _handler->skip(bytes_to_skip, &cursor, _handler->context);
    bytes_skipped = cursor;
    return result;
}

bool IResourceHandler::Read(void *data_out,
                            int bytes_to_read,
                            int &bytes_read,
                            CefRefPtr<CefResourceReadCallback> callback)
{
    int cursor = 0;
    bool result = _handler->read((uint8_t *)data_out, bytes_to_read, &cursor, _handler->context);
    bytes_read = cursor;
    return result;
}

void IResourceHandler::Cancel()
{
    _handler->cancel(_handler->context);
}

ISchemeHandlerFactory::ISchemeHandlerFactory(ICustomSchemeAttributes &attr) : _attr(attr)
{
}

CefRefPtr<CefResourceHandler> ISchemeHandlerFactory::Create(CefRefPtr<CefBrowser> browser,
                                                            CefRefPtr<CefFrame> frame,
                                                            const CefString &scheme_name,
                                                            CefRefPtr<CefRequest> req)
{
    if (_attr.factory == nullptr)
    {
        return nullptr;
    }

    std::string referrer = req->GetReferrerURL().ToString();
    std::string method = req->GetMethod().ToString();
    std::string url = req->GetURL().ToString();

    Request request = {.url = url.c_str(), .method = method.c_str(), .referrer = referrer.c_str()};
    auto handler = _attr.factory->request(&request, _attr.factory->context);
    if (handler == nullptr)
    {
        return nullptr;
    }

    return new IResourceHandler(_attr.factory, handler);
}

IResourceRequestHandler::IResourceRequestHandler(const RequestHandlerFactory *factory) : _factory(factory)
{
}

CefRefPtr<CefResourceHandler> IResourceRequestHandler::GetResourceHandler(CefRefPtr<CefBrowser> browser,
                                                                          CefRefPtr<CefFrame> frame,
                                                                          CefRefPtr<CefRequest> req)
{
    if (_factory == nullptr)
    {
        return nullptr;
    }

    std::string referrer = req->GetReferrerURL().ToString();
    std::string method = req->GetMethod().ToString();
    std::string url = req->GetURL().ToString();

    Request request = {.url = url.c_str(), .method = method.c_str(), .referrer = referrer.c_str()};
    auto handler = _factory->request(&request, _factory->context);
    if (handler == nullptr)
    {
        return nullptr;
    }

    return new IResourceHandler(_factory, handler);
}
