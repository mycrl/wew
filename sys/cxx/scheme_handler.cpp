//
//  scheme_handler.cpp
//  webview
//
//  Created by Mr.Panda on 2023/6/28.
//

#include "scheme_handler.h"

static const std::map<std::string, std::string> MIME_TYPE_MAP = {
    {"html", "text/html"},        {"htm", "text/html"},
    {"css", "text/css"},          {"js", "text/javascript"},
    {"json", "application/json"}, {"jpeg", "image/jpeg"},
    {"jpg", "image/jpeg"},        {"png", "image/png"},
    {"webp", "image/webp"},       {"gif", "image/gif"},
    {"avif", "image/avif"},       {"svg", "image/svg+xml"},
    {"icon", "image/x-icon"},     {"ico", "image/x-icon"},
    {"mp3", "audio/mp3"},         {"webm", "video/webm"},
    {"mp4", "video/mp4"},         {"woff", "application/x-font-woff"},
    {"otf", "font/opentype"},     {"manifest", "text/cache-manifest"} };

const std::string ClientSchemeHandler::FormatMime(std::string& url)
{
    auto iter = MIME_TYPE_MAP.find(url.substr(url.rfind('.') + 1, url.size()));
    return iter != MIME_TYPE_MAP.end() ? iter->second : "text/plain";
}

ClientSchemeHandler::ClientSchemeHandler(std::string dir) : _file_root(dir)
{
}

bool ClientSchemeHandler::Open(CefRefPtr<CefRequest> request,
                               bool& handle_request,
                               CefRefPtr<CefCallback> callback)
{
    DCHECK(!CefCurrentlyOn(TID_UI) && !CefCurrentlyOn(TID_IO));

    _url = request->GetURL();
    _url.erase(0, SchemeName.size() + 3);
    _url.erase(0, SchemeDomain.size() + 1);

    _url = _url.substr(0, _url.rfind('#'));
    _url = _url.substr(0, _url.rfind('?'));

    if (_url[_url.size() - 1] == '/')
    {
        _url.pop_back();
    }

    if (_file_root[_file_root.size() - 1] != '/')
    {
        _file_root.push_back('/');
    }

    if (_file_root.find("\\\\?\\") == 0)
    {
        _file_root.erase(0, 4);
    }

    _url = _file_root + _url;
    printf("== cef scheme path rewrite: %s\n", _url.c_str());

#ifdef WIN32
    int size = MultiByteToWideChar(CP_UTF8, 0, _url.c_str(), -1, NULL, 0);
    wchar_t* buf = new wchar_t[size];
    MultiByteToWideChar(CP_UTF8, 0, _url.c_str(), -1, buf, size);
    LPCWSTR lpcwstr = buf;
    _fd = CreateFileW(lpcwstr, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING,
                      FILE_ATTRIBUTE_NORMAL, NULL);
    delete[] buf;

    if (_fd == INVALID_HANDLE_VALUE)
    {
        return false;
    }
#else
    _fd = fopen(_url.c_str(), "r");
    if (!_fd)
    {
        return false;
    }
#endif

    _mime_type = FormatMime(_url);
    handle_request = false;
    callback->Continue();
    return true;
}

void ClientSchemeHandler::GetResponseHeaders(CefRefPtr<CefResponse> response,
                                             int64_t& response_length,
                                             CefString& redirect_url)
{
    CEF_REQUIRE_IO_THREAD();
    std::lock_guard<std::mutex> lock(_mutex);

#ifdef WIN32
    LARGE_INTEGER file_size;
    GetFileSizeEx(_fd.value(), &file_size);
    _size = file_size.QuadPart;
#else
    struct stat stat_buf;
    int ret = stat(_url.c_str(), &stat_buf);
    _size = ret == 0 ? stat_buf.st_size : -1;
#endif

    response_length = _size;
    response->SetMimeType(_mime_type);
    response->SetStatus(200);
}

bool ClientSchemeHandler::Skip(int64_t bytes_to_skip,
                               int64_t& bytes_skipped,
                               CefRefPtr<CefResourceSkipCallback> callback)
{
    CEF_REQUIRE_IO_THREAD();
    return false;
}

bool ClientSchemeHandler::Read(void* data_out,
                               int bytes_to_read,
                               int& bytes_read,
                               CefRefPtr<CefResourceReadCallback> callback)
{
    DCHECK(!CefCurrentlyOn(TID_UI) && !CefCurrentlyOn(TID_IO));
    std::lock_guard<std::mutex> lock(_mutex);

    if (!_fd.has_value())
    {
        bytes_read = -2;
        return false;
    }

    if (_offset >= _size)
    {
        bytes_read = 0;
        return false;
    }

    size_t need_size = static_cast<size_t>(bytes_to_read);
    size_t chunk_size = std::min(need_size, _size - _offset);
    bool ret = false;

#ifdef WIN32
    DWORD read_size;
    ret = ReadFile(_fd.value(), data_out, chunk_size, &read_size, NULL);
#else
    size_t read_size = fread(data_out, 1, chunk_size, _fd.value());
    ret = true;
#endif

    if (!ret)
    {
        bytes_read = -2;
        return false;
    }

    _offset += static_cast<size_t>(read_size);
    bytes_read = static_cast<int>(read_size);
    return read_size > 0;
}

void ClientSchemeHandler::Cancel()
{
    CEF_REQUIRE_IO_THREAD();

    if (!_fd.has_value())
    {
        return;
    }

#ifdef WIN32
    CloseHandle(_fd.value());
#else
    fclose(_fd.value());
#endif

    _fd = std::nullopt;
    _offset = 0;
    _size = 0;
}

ClientSchemeHandlerFactory::ClientSchemeHandlerFactory(std::string dir) : _dir(dir)
{
}

CefRefPtr<CefResourceHandler> ClientSchemeHandlerFactory::Create(CefRefPtr<CefBrowser> browser,
                                                                 CefRefPtr<CefFrame> frame,
                                                                 const CefString& scheme_name,
                                                                 CefRefPtr<CefRequest> request)
{
    CEF_REQUIRE_IO_THREAD();
    return new ClientSchemeHandler(_dir);
}

void RegisterSchemeHandlerFactory(std::string dir)
{
    CefRegisterSchemeHandlerFactory(WEBVIEW_SCHEME_NAME, WEBVIEW_SCHEME_DOMAIN,
                                    new ClientSchemeHandlerFactory(dir));
}
