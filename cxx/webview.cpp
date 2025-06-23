//
//  webview.cpp
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#include "webview.h"

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
    _resource_request_handler = new IResourceRequestHandler(settings->request_handler_factory);
}
// clang-format on

IWebView::~IWebView()
{
    this->Close();
}

CefRefPtr<CefDragHandler> IWebView::GetDragHandler()
{
    CHECK_REFCOUNTING(nullptr);

    return this;
}

CefRefPtr<CefDisplayHandler> IWebView::GetDisplayHandler()
{
    CHECK_REFCOUNTING(nullptr);

    return this;
}

CefRefPtr<CefLifeSpanHandler> IWebView::GetLifeSpanHandler()
{
    CHECK_REFCOUNTING(nullptr);

    return this;
}

CefRefPtr<CefLoadHandler> IWebView::GetLoadHandler()
{
    CHECK_REFCOUNTING(nullptr);

    return this;
}

CefRefPtr<CefRenderHandler> IWebView::GetRenderHandler()
{
    CHECK_REFCOUNTING(nullptr);

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
    CHECK_REFCOUNTING(nullptr);

    if (_resource_request_handler == nullptr)
    {
        return nullptr;
    }

    return this;
}

void IWebView::OnBeforeContextMenu(CefRefPtr<CefBrowser> browser,
                                   CefRefPtr<CefFrame> frame,
                                   CefRefPtr<CefContextMenuParams> params,
                                   CefRefPtr<CefMenuModel> model)
{
    CHECK_REFCOUNTING();

    if (params->GetTypeFlags() & (CM_TYPEFLAG_SELECTION | CM_TYPEFLAG_EDITABLE))
    {
        return;
    }

    model->Clear();
}

CefRefPtr<CefContextMenuHandler> IWebView::GetContextMenuHandler()
{
    CHECK_REFCOUNTING(nullptr);

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

void IWebView::OnLoadStart(CefRefPtr<CefBrowser> browser, CefRefPtr<CefFrame> frame, TransitionType transition_type)
{
    CHECK_REFCOUNTING();

    _handler.on_state_change(WebViewState::BeforeLoad, _handler.context);
}

void IWebView::OnLoadEnd(CefRefPtr<CefBrowser> browser, CefRefPtr<CefFrame> frame, int httpStatusCode)
{
    CHECK_REFCOUNTING();

    _handler.on_state_change(WebViewState::Loaded, _handler.context);
}

void IWebView::OnLoadError(CefRefPtr<CefBrowser> browser,
                           CefRefPtr<CefFrame> frame,
                           ErrorCode error_code,
                           const CefString &error_text,
                           const CefString &failed_url)
{
    CHECK_REFCOUNTING();

    _handler.on_state_change(WebViewState::LoadError, _handler.context);

    if (error_code == ERR_ABORTED)
    {
        return;
    }
}

void IWebView::OnAfterCreated(CefRefPtr<CefBrowser> browser)
{
    CHECK_REFCOUNTING();

    browser->GetHost()->WasResized();
    _browser = browser;
}

bool IWebView::DoClose(CefRefPtr<CefBrowser> browser)
{
    CHECK_REFCOUNTING(true);

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
    CHECK_REFCOUNTING(false);

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

bool IWebView::OnProcessMessageReceived(CefRefPtr<CefBrowser> browser,
                                        CefRefPtr<CefFrame> frame,
                                        CefProcessId source_process,
                                        CefRefPtr<CefProcessMessage> message)
{
    CHECK_REFCOUNTING(false);

    if (!_browser.has_value())
    {
        return false;
    }

    auto args = message->GetArgumentList();
    std::string payload = args->GetString(0);
    _handler.on_message(payload.c_str(), _handler.context);
    return true;
}

void IWebView::OnTitleChange(CefRefPtr<CefBrowser> browser, const CefString &title)
{
    CHECK_REFCOUNTING();

    std::string value = title.ToString();
    _handler.on_title_change(value.c_str(), _handler.context);
};

void IWebView::OnFullscreenModeChange(CefRefPtr<CefBrowser> browser, bool fullscreen)
{
    CHECK_REFCOUNTING();

    _handler.on_fullscreen_change(fullscreen, _handler.context);
};

void IWebView::OnImeCompositionRangeChanged(CefRefPtr<CefBrowser> browser,
                                            const CefRange &selected_range,
                                            const RectList &character_bounds)
{
    CHECK_REFCOUNTING();

    if (character_bounds.size() == 0)
    {
        return;
    }

    auto first = character_bounds[0];
    _handler.on_ime_rect(first, _handler.context);
}

void IWebView::GetViewRect(CefRefPtr<CefBrowser> browser, CefRect &rect)
{
    CHECK_REFCOUNTING();

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
    CHECK_REFCOUNTING();

    if (buffer == nullptr)
    {
        return;
    }

    _handler.on_frame(buffer, width, height, _handler.context);
}

CefRefPtr<CefResourceRequestHandler> IWebView::GetResourceRequestHandler(CefRefPtr<CefBrowser> browser,
                                                                         CefRefPtr<CefFrame> frame,
                                                                         CefRefPtr<CefRequest> request,
                                                                         bool is_navigation,
                                                                         bool is_download,
                                                                         const CefString &request_initiator,
                                                                         bool &disable_default_handling)
{
    CHECK_REFCOUNTING(nullptr);

    return _resource_request_handler;
}

bool IWebView::GetScreenInfo(CefRefPtr<CefBrowser> browser, CefScreenInfo &info)
{
    CHECK_REFCOUNTING(false);

    info.device_scale_factor = _device_scale_factor;

    return true;
}

void IWebView::SetDevToolsOpenState(bool is_open)
{
    CHECK_REFCOUNTING();

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
    CHECK_REFCOUNTING(nullptr);

    return _browser.has_value() ? _browser.value()->GetHost()->GetWindowHandle() : nullptr;
}

void IWebView::SendMessage(std::string message)
{
    CHECK_REFCOUNTING();

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
    CHECK_REFCOUNTING();

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->CloseBrowser(true);
    _browser = std::nullopt;

    CLOSE_RUNNING;
}

void IWebView::OnIMEComposition(std::string input)
{
    CHECK_REFCOUNTING();

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->ImeCommitText(input, CefRange::InvalidRange(), 0);
}

void IWebView::OnIMESetComposition(std::string input, int x, int y)
{
    CHECK_REFCOUNTING();

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
    CHECK_REFCOUNTING();

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->SendMouseClickEvent(event, button, !pressed, 1);
}

void IWebView::OnMouseMove(cef_mouse_event_t event)
{
    CHECK_REFCOUNTING();

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->SendMouseMoveEvent(event, false);
}

void IWebView::OnMouseWheel(cef_mouse_event_t event, int x, int y)
{
    CHECK_REFCOUNTING();

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->SendMouseWheelEvent(event, x, y);
}

void IWebView::OnKeyboard(cef_key_event_t event)
{
    CHECK_REFCOUNTING();

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->SendKeyEvent(event);
}

void IWebView::OnTouch(cef_touch_event_t event)
{
    CHECK_REFCOUNTING();

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->SendTouchEvent(event);
}

void IWebView::Resize(int width, int height)
{
    CHECK_REFCOUNTING();

    if (!_browser.has_value())
    {
        return;
    }

    _view_rect.width = width;
    _view_rect.height = height;
    _browser.value()->GetHost()->WasResized();
}

void close_webview(void *webview)
{
    assert(webview != nullptr);

    auto view = static_cast<WebView *>(webview);
    view->ref->Close();

    delete view;
}

void webview_mouse_click(void *webview, cef_mouse_event_t event, cef_mouse_button_type_t button, bool pressed)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->OnMouseClick(event, button, pressed);
}

void webview_mouse_wheel(void *webview, cef_mouse_event_t event, int x, int y)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->OnMouseWheel(event, x, y);
}

void webview_mouse_move(void *webview, cef_mouse_event_t event)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->OnMouseMove(event);
}

void webview_keyboard(void *webview, cef_key_event_t event)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->OnKeyboard(event);
}

void webview_touch(void *webview, cef_touch_event_t event)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->OnTouch(event);
}

void webview_ime_composition(void *webview, const char *input)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->OnIMEComposition(input);
}

void webview_ime_set_composition(void *webview, const char *input, int x, int y)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->OnIMESetComposition(input, x, y);
}

void webview_send_message(void *webview, const char *message)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->SendMessage(std::string(message));
}

void webview_set_devtools_state(void *webview, bool is_open)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->SetDevToolsOpenState(is_open);
}

void webview_resize(void *webview, int width, int height)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->Resize(width, height);
}

const void *webview_get_window_handle(void *webview)
{
    assert(webview != nullptr);

    return static_cast<WebView *>(webview)->ref->GetWindowHandle();
}
