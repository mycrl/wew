//
//  request.h
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#ifndef request_h
#define request_h
#pragma once

#include <string>

#include "include/cef_request_handler.h"
#include "include/cef_scheme.h"

#include "wew.h"

struct ICustomSchemeAttributes
{
    std::string name;
    std::string domain;
    const RequestHandlerFactory *factory;
};

class IResourceHandler : public CefResourceHandler
{
  public:
    IResourceHandler(const RequestHandlerFactory *factory, RequestHandler *handler);

    ~IResourceHandler();

    ///
    /// Open the response stream.
    ///
    bool Open(CefRefPtr<CefRequest> request, bool &handle_request, CefRefPtr<CefCallback> callback) override;

    ///
    /// Retrieve response header information.
    ///
    void GetResponseHeaders(CefRefPtr<CefResponse> response, int64_t &response_length, CefString &redirectUrl) override;

    ///
    /// Read response data.
    ///
    bool Skip(int64_t bytes_to_skip, int64_t &bytes_skipped, CefRefPtr<CefResourceSkipCallback> callback) override;

    ///
    /// Read response data.
    ///
    bool Read(void *data_out, int bytes_to_read, int &bytes_read, CefRefPtr<CefResourceReadCallback> callback) override;

    ///
    /// Request processing has been canceled.
    ///
    void Cancel() override;

  private:
    RequestHandler *_handler;
    const RequestHandlerFactory *_factory;

    IMPLEMENT_REFCOUNTING(IResourceHandler);
};

class ISchemeHandlerFactory : public CefSchemeHandlerFactory
{
  public:
    ISchemeHandlerFactory(ICustomSchemeAttributes &attr);

    ///
    /// Return a new scheme handler instance to handle the request.
    ///
    CefRefPtr<CefResourceHandler> Create(CefRefPtr<CefBrowser> browser,
                                         CefRefPtr<CefFrame> frame,
                                         const CefString &scheme_name,
                                         CefRefPtr<CefRequest> request) override;

  private:
    ICustomSchemeAttributes &_attr;

    IMPLEMENT_REFCOUNTING(ISchemeHandlerFactory);
    DISALLOW_COPY_AND_ASSIGN(ISchemeHandlerFactory);
};

class IResourceRequestHandler : public CefResourceRequestHandler
{
  public:
    IResourceRequestHandler(const RequestHandlerFactory *factory);

    ///
    /// Called on the IO thread before a resource is loaded.
    ///
    CefRefPtr<CefResourceHandler> GetResourceHandler(CefRefPtr<CefBrowser> browser,
                                                     CefRefPtr<CefFrame> frame,
                                                     CefRefPtr<CefRequest> request) override;

  private:
    const RequestHandlerFactory *_factory = nullptr;

    IMPLEMENT_REFCOUNTING(IResourceRequestHandler);
    DISALLOW_COPY_AND_ASSIGN(IResourceRequestHandler);
};

#endif /* request_h */
