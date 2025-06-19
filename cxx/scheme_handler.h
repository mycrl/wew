//
//  scheme_handler.h
//  webview
//
//  Created by Mr.Panda on 2023/6/28.
//

#ifndef LIBWEBVIEW_SCHEME_HANDLER_H
#define LIBWEBVIEW_SCHEME_HANDLER_H
#pragma once

#include <stdio.h>

#include <mutex>
#include <optional>

#include "include/cef_app.h"
#include "include/wrapper/cef_helpers.h"

#ifdef WIN32
#include "windows.h"
#else
#include <sys/stat.h>
#endif

#ifndef WEBVIEW_SCHEME_NAME
#define WEBVIEW_SCHEME_NAME "webview"
#endif

#ifndef WEBVIEW_SCHEME_DOMAIN
#define WEBVIEW_SCHEME_DOMAIN ""
#endif

class ClientSchemeHandler : public CefResourceHandler
{
public:
    static const std::string FormatMime(std::string& url);
    
    ClientSchemeHandler(std::string dir);
    ~ClientSchemeHandler()
    {
        Cancel();
    }
    
    /* CefResourceHandler */
    
    bool Open(CefRefPtr<CefRequest> request,
              bool& handle_request,
              CefRefPtr<CefCallback> callback) override;
    void GetResponseHeaders(CefRefPtr<CefResponse> response,
                            int64_t& response_length,
                            CefString& redirect_url) override;
    virtual bool Skip(int64_t bytes_to_skip,
                      int64_t& bytes_skipped,
                      CefRefPtr<CefResourceSkipCallback> callback) override;
    bool Read(void* data_out,
              int bytes_to_read,
              int& bytes_read,
              CefRefPtr<CefResourceReadCallback> callback) override;
    void Cancel() override;
    
    const std::string SchemeName = WEBVIEW_SCHEME_NAME;
    const std::string SchemeDomain = WEBVIEW_SCHEME_DOMAIN;
    
private:
    std::string _file_root;
    std::string _mime_type = "";
    size_t _offset = 0;
    size_t _size = 0;
    std::mutex _mutex;
    std::string _url;
#ifdef WIN32
    std::optional<HANDLE> _fd = std::nullopt;
#else
    std::optional<FILE*> _fd = std::nullopt;
#endif
    
    IMPLEMENT_REFCOUNTING(ClientSchemeHandler);
    DISALLOW_COPY_AND_ASSIGN(ClientSchemeHandler);
};

class ClientSchemeHandlerFactory : public CefSchemeHandlerFactory
{
public:
    ClientSchemeHandlerFactory(std::string dir);
    
    // Return a new scheme handler instance to handle the request.
    CefRefPtr<CefResourceHandler> Create(CefRefPtr<CefBrowser> browser,
                                         CefRefPtr<CefFrame> frame,
                                         const CefString& scheme_name,
                                         CefRefPtr<CefRequest> request) override;
    
private:
    std::string _dir;
    
    IMPLEMENT_REFCOUNTING(ClientSchemeHandlerFactory);
    DISALLOW_COPY_AND_ASSIGN(ClientSchemeHandlerFactory);
};

void RegisterSchemeHandlerFactory(std::string dir);

#endif  // LIBWEBVIEW_SCHEME_HANDLER_H
