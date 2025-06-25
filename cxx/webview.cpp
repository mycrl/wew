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

/* CefClient */

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

CefRefPtr<CefContextMenuHandler> IWebView::GetContextMenuHandler()
{
    CHECK_REFCOUNTING(nullptr);

    return this;
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

/* CefContextMenuHandler */

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

bool IWebView::OnContextMenuCommand(CefRefPtr<CefBrowser> browser,
                                    CefRefPtr<CefFrame> frame,
                                    CefRefPtr<CefContextMenuParams> params,
                                    int command_id,
                                    EventFlags event_flags)
{
    return false;
};

/* CefLoadHandler */

void IWebView::OnLoadStart(CefRefPtr<CefBrowser> browser, CefRefPtr<CefFrame> frame, TransitionType transition_type)
{
    CHECK_REFCOUNTING();

    _handler.on_state_change(WebViewState::WEBVIEW_BEFORE_LOAD, _handler.context);
}

void IWebView::OnLoadEnd(CefRefPtr<CefBrowser> browser, CefRefPtr<CefFrame> frame, int httpStatusCode)
{
    CHECK_REFCOUNTING();

    _handler.on_state_change(WebViewState::WEBVIEW_LOADED, _handler.context);
    browser->GetHost()->SetFocus(true);
}

void IWebView::OnLoadError(CefRefPtr<CefBrowser> browser,
                           CefRefPtr<CefFrame> frame,
                           ErrorCode error_code,
                           const CefString &error_text,
                           const CefString &failed_url)
{
    CHECK_REFCOUNTING();

    _handler.on_state_change(WebViewState::WEBVIEW_LOAD_ERROR, _handler.context);

    if (error_code == ERR_ABORTED)
    {
        return;
    }
}

/* CefLifeSpanHandler */

void IWebView::OnAfterCreated(CefRefPtr<CefBrowser> browser)
{
    CHECK_REFCOUNTING();

    browser->GetHost()->WasResized();
    _browser = browser;
}

bool IWebView::DoClose(CefRefPtr<CefBrowser> browser)
{
    CHECK_REFCOUNTING(true);

    _handler.on_state_change(WebViewState::WEBVIEW_REQUEST_CLOSE, _handler.context);

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

void IWebView::OnBeforeClose(CefRefPtr<CefBrowser> browser)
{
    _handler.on_state_change(WebViewState::WEBVIEW_CLOSE, _handler.context);
    _browser = std::nullopt;
}

/* CefDragHandler */

bool IWebView::OnDragEnter(CefRefPtr<CefBrowser> browser,
                           CefRefPtr<CefDragData> dragData,
                           CefDragHandler::DragOperationsMask mask)
{
    return true;
}

/* CefDisplayHandler */

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

/* CefRenderHandler */

bool IWebView::GetScreenInfo(CefRefPtr<CefBrowser> browser, CefScreenInfo &info)
{
    CHECK_REFCOUNTING(false);

    info.device_scale_factor = _device_scale_factor;

    return true;
}

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

    Rect rect;
    rect.x = first.x;
    rect.y = first.y;
    rect.width = first.width;
    rect.height = first.height;

    _handler.on_ime_rect(rect, _handler.context);
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

/* CefRequestHandler */

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

/* custom impl */

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

void IWebView::SetFocus(bool enable)
{
    CHECK_REFCOUNTING();

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->SetFocus(enable);
}

void close_webview(void *webview)
{
    assert(webview != nullptr);

    auto view = static_cast<WebView *>(webview);
    view->ref->Close();

    delete view;
}

void webview_mouse_click(void *webview, MouseEvent event, MouseButton button, bool pressed)
{
    assert(webview != nullptr);

    CefMouseEvent cef_event;
    cef_event.x = event.x;
    cef_event.y = event.y;
    cef_event.modifiers = event.modifiers;

    auto cef_button = static_cast<cef_mouse_button_type_t>(static_cast<int>(button));

    static_cast<WebView *>(webview)->ref->OnMouseClick(cef_event, cef_button, pressed);
}

void webview_mouse_wheel(void *webview, MouseEvent event, int x, int y)
{
    assert(webview != nullptr);

    CefMouseEvent cef_event;
    cef_event.x = event.x;
    cef_event.y = event.y;
    cef_event.modifiers = event.modifiers;

    static_cast<WebView *>(webview)->ref->OnMouseWheel(cef_event, x, y);
}

void webview_mouse_move(void *webview, MouseEvent event)
{
    assert(webview != nullptr);

    CefMouseEvent cef_event;
    cef_event.x = event.x;
    cef_event.y = event.y;
    cef_event.modifiers = event.modifiers;

    static_cast<WebView *>(webview)->ref->OnMouseMove(cef_event);
}

void webview_keyboard(void *webview, KeyEvent event)
{
    assert(webview != nullptr);

    CefKeyEvent cef_event;
    cef_event.modifiers = event.modifiers;
    cef_event.native_key_code = event.native_key_code;
    cef_event.windows_key_code = event.windows_key_code;
    cef_event.character = event.character;
    cef_event.unmodified_character = event.unmodified_character;
    cef_event.is_system_key = event.is_system_key;
    cef_event.focus_on_editable_field = event.focus_on_editable_field;
    cef_event.type = static_cast<cef_key_event_type_t>(static_cast<int>(event.type));

    static_cast<WebView *>(webview)->ref->OnKeyboard(cef_event);
}

void webview_touch(void *webview, TouchEvent event)
{
    assert(webview != nullptr);

    CefTouchEvent cef_event;
    cef_event.id = event.id;
    cef_event.x = event.x;
    cef_event.y = event.y;
    cef_event.radius_x = event.radius_x;
    cef_event.radius_y = event.radius_y;
    cef_event.pressure = event.pressure;
    cef_event.modifiers = event.modifiers;
    cef_event.rotation_angle = event.rotation_angle;
    cef_event.type = static_cast<cef_touch_event_type_t>(static_cast<int>(event.type));
    cef_event.pointer_type = static_cast<cef_pointer_type_t>(static_cast<int>(event.pointer_type));

    static_cast<WebView *>(webview)->ref->OnTouch(cef_event);
}

void webview_ime_composition(void *webview, const char *input)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->OnIMEComposition(std::string(input));
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

void webview_set_focus(void *webview, bool enable)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->SetFocus(enable);
}
