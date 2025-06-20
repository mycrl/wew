//
//  webview.cpp
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#include "webview.h"
#include "request_handler.h"

void close_webview(void *webview_ptr)
{
    assert(webview_ptr != nullptr);

    auto webview = static_cast<WebView *>(webview_ptr);
    webview->ref->Close();
    delete webview;
}

void webview_mouse_click(void *webview_ptr, cef_mouse_event_t event, cef_mouse_button_type_t button, bool pressed)
{
    assert(webview_ptr != nullptr);

    auto webview = static_cast<WebView *>(webview_ptr);
    webview->ref->OnMouseClick(event, button, pressed);
}

void webview_mouse_wheel(void *webview_ptr, cef_mouse_event_t event, int x, int y)
{
    assert(webview_ptr != nullptr);

    auto webview = static_cast<WebView *>(webview_ptr);
    webview->ref->OnMouseWheel(event, x, y);
}

void webview_mouse_move(void *webview_ptr, cef_mouse_event_t event)
{
    assert(webview_ptr != nullptr);

    auto webview = static_cast<WebView *>(webview_ptr);
    webview->ref->OnMouseMove(event);
}

void webview_keyboard(void *webview_ptr, cef_key_event_t event)
{
    assert(webview_ptr != nullptr);

    auto webview = static_cast<WebView *>(webview_ptr);
    webview->ref->OnKeyboard(event);
}

void webview_touch(void *webview_ptr, cef_touch_event_t event)
{
    assert(webview_ptr != nullptr);

    auto webview = static_cast<WebView *>(webview_ptr);
    webview->ref->OnTouch(event);
}

void webview_ime_composition(void *webview_ptr, const char *input)
{
    assert(webview_ptr != nullptr);

    auto webview = static_cast<WebView *>(webview_ptr);
    webview->ref->OnIMEComposition(input);
}

void webview_ime_set_composition(void *webview_ptr, const char *input, int x, int y)
{
    assert(webview_ptr != nullptr);

    auto webview = static_cast<WebView *>(webview_ptr);
    webview->ref->OnIMESetComposition(input, x, y);
}

void webview_send_message(void *webview_ptr, const char *message)
{
    assert(webview_ptr != nullptr);

    auto webview = static_cast<WebView *>(webview_ptr);
    webview->ref->SendMessage(std::string(message));
}

void webview_set_devtools_state(void *webview_ptr, bool is_open)
{
    assert(webview_ptr != nullptr);

    auto webview = static_cast<WebView *>(webview_ptr);
    webview->ref->SetDevToolsOpenState(is_open);
}

void webview_resize(void *webview_ptr, int width, int height)
{
    assert(webview_ptr != nullptr);

    auto webview = static_cast<WebView *>(webview_ptr);
    webview->ref->Resize(width, height);
}

const void *webview_get_window_handle(void *webview_ptr)
{
    assert(webview_ptr != nullptr);

    auto webview = static_cast<WebView *>(webview_ptr);
    return webview->ref->GetWindowHandle();
}

void webview_set_request_handler(void *webview_ptr, ResourceRequestHandler *handler)
{
    assert(webview_ptr != nullptr);

    auto webview = static_cast<WebView *>(webview_ptr);
    webview->ref->SetRequestHandler(handler);
}

// clang-format off
IWebView::IWebView(CefSettings &cef_settings, 
                   const WebViewSettings *settings, 
                   WebViewHandler handler)
    : _cef_settings(cef_settings)
    , _handler(handler)
{
    _view_rect.width = settings->width;
    _view_rect.height = settings->height;
    _device_scale_factor = settings->device_scale_factor;
}
// clang-format on

IWebView::~IWebView()
{
    this->Close();
}

CefRefPtr<CefDragHandler> IWebView::GetDragHandler()
{
    return this;
}

void IWebView::OnBeforeContextMenu(CefRefPtr<CefBrowser> browser,
                                   CefRefPtr<CefFrame> frame,
                                   CefRefPtr<CefContextMenuParams> params,
                                   CefRefPtr<CefMenuModel> model)
{
    if (params->GetTypeFlags() & (CM_TYPEFLAG_SELECTION | CM_TYPEFLAG_EDITABLE))
    {
        return;
    }

    model->Clear();
}

CefRefPtr<CefContextMenuHandler> IWebView::GetContextMenuHandler()
{
    return this;
}

bool IWebView::OnContextMenuCommand(CefRefPtr<CefBrowser> browser,
                                    CefRefPtr<CefFrame> frame,
                                    CefRefPtr<CefContextMenuParams> params,
                                    int command_id,
                                    EventFlags event_flags)
{
    return false;
};

CefRefPtr<CefDisplayHandler> IWebView::GetDisplayHandler()
{
    if (_is_closed)
    {
        return nullptr;
    }

    return this;
}

CefRefPtr<CefLifeSpanHandler> IWebView::GetLifeSpanHandler()
{
    if (_is_closed)
    {
        return nullptr;
    }

    return this;
}

CefRefPtr<CefLoadHandler> IWebView::GetLoadHandler()
{
    if (_is_closed)
    {
        return nullptr;
    }

    return this;
}

CefRefPtr<CefRenderHandler> IWebView::GetRenderHandler()
{
    if (_cef_settings.windowless_rendering_enabled)
    {
        return this;
    }
    else
    {
        return nullptr;
    }
}

CefRefPtr<CefRequestHandler> IWebView::GetRequestHandler()
{
    if (_is_closed)
    {
        return nullptr;
    }

    if (_request_handler == nullptr)
    {
        return nullptr;
    }

    return this;
}

void IWebView::OnLoadStart(CefRefPtr<CefBrowser> browser, CefRefPtr<CefFrame> frame, TransitionType transition_type)
{
    if (_is_closed)
    {
        return;
    }

    _handler.on_state_change(WebViewState::BeforeLoad, _handler.context);
}

void IWebView::OnLoadEnd(CefRefPtr<CefBrowser> browser, CefRefPtr<CefFrame> frame, int httpStatusCode)
{
    if (_is_closed)
    {
        return;
    }

    _handler.on_state_change(WebViewState::Loaded, _handler.context);
}

void IWebView::OnLoadError(CefRefPtr<CefBrowser> browser,
                           CefRefPtr<CefFrame> frame,
                           ErrorCode error_code,
                           const CefString &error_text,
                           const CefString &failed_url)
{
    if (_is_closed)
    {
        return;
    }

    _handler.on_state_change(WebViewState::LoadError, _handler.context);

    if (error_code == ERR_ABORTED)
    {
        return;
    }
}

void IWebView::OnAfterCreated(CefRefPtr<CefBrowser> browser)
{
    if (_is_closed)
    {
        return;
    }

    browser->GetHost()->WasResized();
    _browser = browser;
}

bool IWebView::DoClose(CefRefPtr<CefBrowser> browser)
{
    _handler.on_state_change(WebViewState::RequestClose, _handler.context);

    return false;
}

bool IWebView::OnBeforePopup(CefRefPtr<CefBrowser> browser,
                             CefRefPtr<CefFrame> frame,
                             int popup_id,
                             const CefString &target_url,
                             const CefString &target_frame_name,
                             CefLifeSpanHandler::WindowOpenDisposition target_disposition,
                             bool user_gesture,
                             const CefPopupFeatures &popupFeatures,
                             CefWindowInfo &windowInfo,
                             CefRefPtr<CefClient> &client,
                             CefBrowserSettings &settings,
                             CefRefPtr<CefDictionaryValue> &extra_info,
                             bool *no_javascript_access)
{
    browser->GetMainFrame()->LoadURL(target_url);

    return true;
}

bool IWebView::OnDragEnter(CefRefPtr<CefBrowser> browser,
                           CefRefPtr<CefDragData> dragData,
                           CefDragHandler::DragOperationsMask mask)
{
    return true;
}

void IWebView::OnBeforeClose(CefRefPtr<CefBrowser> browser)
{
    _handler.on_state_change(WebViewState::Close, _handler.context);
    _browser = std::nullopt;
}

void IWebView::SetDevToolsOpenState(bool is_open)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    if (is_open)
    {
        _browser.value()->GetHost()->ShowDevTools(CefWindowInfo(), nullptr, CefBrowserSettings(), CefPoint());
    }
    else
    {
        _browser.value()->GetHost()->CloseDevTools();
    }
}

const void *IWebView::GetWindowHandle()
{
    return _browser.has_value() ? _browser.value()->GetHost()->GetWindowHandle() : nullptr;
}

bool IWebView::OnProcessMessageReceived(CefRefPtr<CefBrowser> browser,
                                        CefRefPtr<CefFrame> frame,
                                        CefProcessId source_process,
                                        CefRefPtr<CefProcessMessage> message)
{
    if (_is_closed)
    {
        return false;
    }

    if (!_browser.has_value())
    {
        return false;
    }

    auto args = message->GetArgumentList();
    std::string payload = args->GetString(0);
    _handler.on_message(payload.c_str(), _handler.context);
    return true;
}

void IWebView::SendMessage(std::string message)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    auto msg = CefProcessMessage::Create("MESSAGE_TRANSPORT");
    CefRefPtr<CefListValue> args = msg->GetArgumentList();
    args->SetSize(1);
    args->SetString(0, message);
    _browser.value()->GetMainFrame()->SendProcessMessage(PID_RENDERER, msg);
}

void IWebView::Close()
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->CloseBrowser(true);
    _browser = std::nullopt;
    _is_closed = true;
}

void IWebView::OnTitleChange(CefRefPtr<CefBrowser> browser, const CefString &title)
{
    if (_is_closed)
    {
        return;
    }

    _handler.on_title_change(title.ToString().c_str(), _handler.context);
};

void IWebView::OnFullscreenModeChange(CefRefPtr<CefBrowser> browser, bool fullscreen)
{
    if (_is_closed)
    {
        return;
    }

    _handler.on_fullscreen_change(fullscreen, _handler.context);
};

void IWebView::OnImeCompositionRangeChanged(CefRefPtr<CefBrowser> browser,
                                            const CefRange &selected_range,
                                            const RectList &character_bounds)
{
    if (_is_closed)
    {
        return;
    }

    if (character_bounds.size() == 0)
    {
        return;
    }

    auto first = character_bounds[0];
    _handler.on_ime_rect(first, _handler.context);
}

void IWebView::GetViewRect(CefRefPtr<CefBrowser> browser, CefRect &rect)
{
    if (_is_closed)
    {
        return;
    }

    rect.width = _view_rect.width;
    rect.height = _view_rect.height;
}

void IWebView::OnPaint(CefRefPtr<CefBrowser> browser,
                       PaintElementType type,
                       const RectList &dirtyRects,
                       const void *buffer, // BGRA32
                       int width,
                       int height)
{
    if (_is_closed)
    {
        return;
    }

    if (buffer == nullptr)
    {
        return;
    }

    _handler.on_frame(buffer, width, height, _handler.context);
}

bool IWebView::GetScreenInfo(CefRefPtr<CefBrowser> browser, CefScreenInfo &info)
{
    if (_is_closed)
    {
        return false;
    }

    info.device_scale_factor = _device_scale_factor;

    return false;
}

void IWebView::OnIMEComposition(std::string input)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->ImeCommitText(input, CefRange::InvalidRange(), 0);
}

void IWebView::OnIMESetComposition(std::string input, int x, int y)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    CefCompositionUnderline line;
    line.style = CEF_CUS_DASH;
    line.range = CefRange(0, y);

    _browser.value()->GetHost()->ImeSetComposition(input, {line}, CefRange::InvalidRange(), CefRange(x, y));
}
void IWebView::OnMouseClick(cef_mouse_event_t event, cef_mouse_button_type_t button, bool pressed)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->SendMouseClickEvent(event, button, !pressed, 1);
}

void IWebView::OnMouseMove(cef_mouse_event_t event)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->SendMouseMoveEvent(event, false);
}

void IWebView::OnMouseWheel(cef_mouse_event_t event, int x, int y)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->SendMouseWheelEvent(event, x, y);
}

void IWebView::OnKeyboard(cef_key_event_t event)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->SendKeyEvent(event);
}

void IWebView::OnTouch(cef_touch_event_t event)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->SendTouchEvent(event);
}

void IWebView::Resize(int width, int height)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    _view_rect.width = width;
    _view_rect.height = height;
    _browser.value()->GetHost()->WasResized();
}

void IWebView::SetRequestHandler(ResourceRequestHandler *handler)
{
    _request_handler = handler;
}

CefRefPtr<CefResourceRequestHandler> IWebView::GetResourceRequestHandler(CefRefPtr<CefBrowser> browser,
                                                                         CefRefPtr<CefFrame> frame,
                                                                         CefRefPtr<CefRequest> req,
                                                                         bool is_navigation,
                                                                         bool is_download,
                                                                         const CefString &request_initiator,
                                                                         bool &disable_default_handling)
{
    if (_is_closed)
    {
        return nullptr;
    }

    if (_request_handler == nullptr)
    {
        return nullptr;
    }

    ResourceRequest request;
    request.url = req->GetURL().ToString().c_str();
    request.method = req->GetMethod().ToString().c_str();
    request.referrer = req->GetReferrerURL().ToString().c_str();

    auto handler = _request_handler->create_resource_handler(&request, _handler.context);
    if (handler == nullptr)
    {
        return nullptr;
    }

    return new IRequestHandler(_request_handler, handler);
}
