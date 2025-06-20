//
//  request_handler.cpp
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#include "request_handler.h"

// clang-format off
IResourceHandler::IResourceHandler(ResourceRequestHandler *request_handler, ResourceHandler *handler)
    : _handler(handler)
    , _request_handler(request_handler)
{
}
// clang-format on

IResourceHandler::~IResourceHandler()
{
    _handler->destroy(_handler->context);
    _request_handler->destroy_resource_handler(_handler);
}

bool IResourceHandler::Open(CefRefPtr<CefRequest> request, bool &handle_request, CefRefPtr<CefCallback> callback)
{
    bool result = _handler->open(_handler->context);
    handle_request = !result;
    return result;
}

void IResourceHandler::GetResponseHeaders(CefRefPtr<CefResponse> response,
                                          int64_t &response_length,
                                          CefString &redirectUrl)
{
    ResourceResponse res;
    _handler->get_response(&res, _handler->context);
    response->SetStatus(res.status_code);
    response->SetMimeType(res.mime_type);
    response_length = res.content_length;
}

bool IResourceHandler::Skip(int64_t bytes_to_skip, int64_t &bytes_skipped, CefRefPtr<CefResourceSkipCallback> callback)
{
    size_t cursor = 0;
    bool result = _handler->skip(bytes_to_skip, &cursor, _handler->context);
    bytes_skipped = result ? cursor : -2;
    return result;
}

bool IResourceHandler::Read(void *data_out,
                            int bytes_to_read,
                            int &bytes_read,
                            CefRefPtr<CefResourceReadCallback> callback)
{
    size_t cursor = 0;
    bool result = _handler->read(data_out, bytes_to_read, &cursor, _handler->context);
    bytes_read = result ? cursor : -2;
    return result;
}

void IResourceHandler::Cancel()
{
    _handler->cancel(_handler->context);
}

IRequestHandler::IRequestHandler(ResourceRequestHandler *request_handler, ResourceHandler *handler)
    : _handler(new IResourceHandler(request_handler, handler))
{
}

CefRefPtr<CefResourceHandler> IRequestHandler::GetResourceHandler(CefRefPtr<CefBrowser> browser,
                                                                  CefRefPtr<CefFrame> frame,
                                                                  CefRefPtr<CefRequest> request)
{
    return _handler;
}
