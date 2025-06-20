//
//  request_handler.h
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#ifndef request_handler_h
#define request_handler_h
#pragma once

#include "include/cef_request_handler.h"

#include "library.h"

class IResourceHandler : public CefResourceHandler
{
  public:
    IResourceHandler(ResourceRequestHandler *request_handler, ResourceHandler *handler);

    ~IResourceHandler();

    ///
    /// Open the response stream. To handle the request immediately set
    /// |handle_request| to true and return true. To decide at a later time set
    /// |handle_request| to false, return true, and execute |callback| to continue
    /// or cancel the request. To cancel the request immediately set
    /// |handle_request| to true and return false. This method will be called in
    /// sequence but not from a dedicated thread. For backwards compatibility set
    /// |handle_request| to false and return false and the ProcessRequest method
    /// will be called.
    ///
    virtual bool Open(CefRefPtr<CefRequest> request, bool &handle_request, CefRefPtr<CefCallback> callback) override;

    ///
    /// Retrieve response header information. If the response length is not known
    /// set |response_length| to -1 and ReadResponse() will be called until it
    /// returns false. If the response length is known set |response_length|
    /// to a positive value and ReadResponse() will be called until it returns
    /// false or the specified number of bytes have been read. Use the |response|
    /// object to set the mime type, http status code and other optional header
    /// values. To redirect the request to a new URL set |redirectUrl| to the new
    /// URL. |redirectUrl| can be either a relative or fully qualified URL.
    /// It is also possible to set |response| to a redirect http status code
    /// and pass the new URL via a Location header. Likewise with |redirectUrl| it
    /// is valid to set a relative or fully qualified URL as the Location header
    /// value. If an error occured while setting up the request you can call
    /// SetError() on |response| to indicate the error condition.
    ///
    virtual void GetResponseHeaders(CefRefPtr<CefResponse> response,
                                    int64_t &response_length,
                                    CefString &redirectUrl) override;

    ///
    /// Skip response data when requested by a Range header. Skip over and discard
    /// |bytes_to_skip| bytes of response data. If data is available immediately
    /// set |bytes_skipped| to the number of bytes skipped and return true. To
    /// read the data at a later time set |bytes_skipped| to 0, return true and
    /// execute |callback| when the data is available. To indicate failure set
    /// |bytes_skipped| to < 0 (e.g. -2 for ERR_FAILED) and return false. This
    /// method will be called in sequence but not from a dedicated thread.
    ///
    virtual bool Skip(int64_t bytes_to_skip,
                      int64_t &bytes_skipped,
                      CefRefPtr<CefResourceSkipCallback> callback) override;

    ///
    /// Read response data. If data is available immediately copy up to
    /// |bytes_to_read| bytes into |data_out|, set |bytes_read| to the number of
    /// bytes copied, and return true. To read the data at a later time keep a
    /// pointer to |data_out|, set |bytes_read| to 0, return true and execute
    /// |callback| when the data is available (|data_out| will remain valid until
    /// the callback is executed). To indicate response completion set
    /// |bytes_read| to 0 and return false. To indicate failure set |bytes_read|
    /// to < 0 (e.g. -2 for ERR_FAILED) and return false. This method will be
    /// called in sequence but not from a dedicated thread. For backwards
    /// compatibility set |bytes_read| to -1 and return false and the ReadResponse
    /// method will be called.
    ///
    virtual bool Read(void *data_out,
                      int bytes_to_read,
                      int &bytes_read,
                      CefRefPtr<CefResourceReadCallback> callback) override;

    ///
    /// Request processing has been canceled.
    ///
    virtual void Cancel() override;

  private:
    ResourceHandler *_handler;
    ResourceRequestHandler *_request_handler;

    IMPLEMENT_REFCOUNTING(IResourceHandler);
};

class IRequestHandler : public CefRequestHandler, public CefResourceRequestHandler
{
  public:
    IRequestHandler(ResourceRequestHandler *request_handler, ResourceHandler *handler);

    ///
    /// Called on the IO thread before a resource is loaded.
    ///
    virtual CefRefPtr<CefResourceHandler> GetResourceHandler(CefRefPtr<CefBrowser> browser,
                                                             CefRefPtr<CefFrame> frame,
                                                             CefRefPtr<CefRequest> request) override;

  private:
    CefRefPtr<CefResourceHandler> _handler;

    IMPLEMENT_REFCOUNTING(IRequestHandler);
};

#endif /* request_handler_h */
