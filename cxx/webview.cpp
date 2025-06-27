//
//  webview.cpp
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#include "webview.h"

/* CefContextMenuHandler */

void IWebViewContextMenu::OnBeforeContextMenu(CefRefPtr<CefBrowser> browser,
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

bool IWebViewContextMenu::OnContextMenuCommand(CefRefPtr<CefBrowser> browser,
                                               CefRefPtr<CefFrame> frame,
                                               CefRefPtr<CefContextMenuParams> params,
                                               int command_id,
                                               EventFlags event_flags)
{
    return false;
};

/* CefLoadHandler */

IWebViewLoad::IWebViewLoad(WebViewHandler &handler) : _handler(handler)
{
}

void IWebViewLoad::OnLoadStart(CefRefPtr<CefBrowser> browser, CefRefPtr<CefFrame> frame, TransitionType transition_type)
{
    _handler.on_state_change(WebViewState::WEBVIEW_BEFORE_LOAD, _handler.context);
}

void IWebViewLoad::OnLoadEnd(CefRefPtr<CefBrowser> browser, CefRefPtr<CefFrame> frame, int httpStatusCode)
{
    _handler.on_state_change(WebViewState::WEBVIEW_LOADED, _handler.context);
    browser->GetHost()->SetFocus(true);
}

void IWebViewLoad::OnLoadError(CefRefPtr<CefBrowser> browser,
                               CefRefPtr<CefFrame> frame,
                               ErrorCode error_code,
                               const CefString &error_text,
                               const CefString &failed_url)
{
    _handler.on_state_change(WebViewState::WEBVIEW_LOAD_ERROR, _handler.context);
}

/* CefLifeSpanHandler */

// clang-format off
IWebViewLifeSpan::IWebViewLifeSpan(std::optional<CefRefPtr<CefBrowser>> &browser, WebViewHandler &handler)
    : _handler(handler)
    , _browser(browser)
{
}
// clang-format on

void IWebViewLifeSpan::OnAfterCreated(CefRefPtr<CefBrowser> browser)
{
    _browser = browser;

    browser->GetHost()->WasResized();
}

bool IWebViewLifeSpan::DoClose(CefRefPtr<CefBrowser> browser)
{
    _handler.on_state_change(WebViewState::WEBVIEW_REQUEST_CLOSE, _handler.context);

    return false;
}

bool IWebViewLifeSpan::OnBeforePopup(CefRefPtr<CefBrowser> browser,
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

void IWebViewLifeSpan::OnBeforeClose(CefRefPtr<CefBrowser> browser)
{
    _browser = std::nullopt;

    _handler.on_state_change(WebViewState::WEBVIEW_CLOSE, _handler.context);
}

/* CefDragHandler */

bool IWebViewDrag::OnDragEnter(CefRefPtr<CefBrowser> browser,
                               CefRefPtr<CefDragData> dragData,
                               CefDragHandler::DragOperationsMask mask)
{
    return true;
}

/* CefDisplayHandler */

IWebViewDisplay::IWebViewDisplay(WebViewHandler &handler) : _handler(handler)
{
}

void IWebViewDisplay::OnTitleChange(CefRefPtr<CefBrowser> browser, const CefString &title)
{
    std::string value = title.ToString();
    _handler.on_title_change(value.c_str(), _handler.context);
};

void IWebViewDisplay::OnFullscreenModeChange(CefRefPtr<CefBrowser> browser, bool fullscreen)
{
    _handler.on_fullscreen_change(fullscreen, _handler.context);
};

/* CefRenderHandler */

// clang-format off
IWebViewRender::IWebViewRender(const WebViewSettings *settings, WebViewHandler &handler)
    : _handler(handler)
    , _device_scale_factor(settings->device_scale_factor)
{
    assert(settings != nullptr);

    _view_rect.width = settings->width;
    _view_rect.height = settings->height;
}
// clang-format on

bool IWebViewRender::GetScreenInfo(CefRefPtr<CefBrowser> browser, CefScreenInfo &info)
{
    info.device_scale_factor = _device_scale_factor;

    return true;
}

void IWebViewRender::OnImeCompositionRangeChanged(CefRefPtr<CefBrowser> browser,
                                                  const CefRange &selected_range,
                                                  const RectList &character_bounds)
{
    if (character_bounds.size() == 0)
    {
        return;
    }

    auto first_rect = character_bounds[0];

    Rect rect;
    rect.x = first_rect.x;
    rect.y = first_rect.y;
    rect.width = first_rect.width;
    rect.height = first_rect.height;

    _handler.on_ime_rect(rect, _handler.context);
}

void IWebViewRender::GetViewRect(CefRefPtr<CefBrowser> browser, CefRect &rect)
{
    rect.x = _view_rect.x;
    rect.y = _view_rect.y;
    rect.width = _view_rect.width;
    rect.height = _view_rect.height;
}

void IWebViewRender::OnPaint(CefRefPtr<CefBrowser> browser,
                             PaintElementType type,
                             const RectList &dirtyRects,
                             const void *buffer, // BGRA32
                             int width,
                             int height)
{
    if (buffer == nullptr)
    {
        return;
    }

    bool is_popup = type == PaintElementType::PET_POPUP;

    auto first_rect = dirtyRects[0];
    _texture_rect.width = is_popup ? first_rect.width : width;
    _texture_rect.height = is_popup ? first_rect.height : height;
    _texture_rect.x = is_popup ? _popup_rect.x : 0;
    _texture_rect.y = is_popup ? _popup_rect.y : 0;

    _handler.on_frame(buffer, &_texture_rect, _handler.context);
}

void IWebViewRender::OnPopupSize(CefRefPtr<CefBrowser> browser, const CefRect &rect)
{
    _popup_rect.x = rect.x;
    _popup_rect.y = rect.y;
    _popup_rect.width = rect.width;
    _popup_rect.height = rect.height;
}

void IWebViewRender::Resize(int width, int height)
{
    _view_rect.width = width;
    _view_rect.height = height;
}

/* CefRequestHandler */

IWebViewRequest::IWebViewRequest(const WebViewSettings *settings)
    : _handler(new IResourceRequestHandler(settings->request_handler_factory))
{
    assert(settings != nullptr);
}

CefRefPtr<CefResourceRequestHandler> IWebViewRequest::GetResourceRequestHandler(CefRefPtr<CefBrowser> browser,
                                                                                CefRefPtr<CefFrame> frame,
                                                                                CefRefPtr<CefRequest> request,
                                                                                bool is_navigation,
                                                                                bool is_download,
                                                                                const CefString &request_initiator,
                                                                                bool &disable_default_handling)
{
    return _handler;
}

/* IWebView */

IWebView::IWebView(CefSettings &cef_settings, const WebViewSettings *settings, WebViewHandler handler)
    : _handler(handler)
{
    assert(settings != nullptr);

    _drag_handler = new IWebViewDrag();
    _load_handler = new IWebViewLoad(_handler);
    _display_handler = new IWebViewDisplay(_handler);
    _life_span_handler = new IWebViewLifeSpan(_browser, _handler);
    _context_menu_handler = new IWebViewContextMenu();

    if (cef_settings.windowless_rendering_enabled)
    {
        _render_handler = new IWebViewRender(settings, _handler);
    }

    if (settings->request_handler_factory)
    {
        _request_handler = new IWebViewRequest(settings);
    }
}

IWebView::~IWebView()
{
    this->Close();
}

CefRefPtr<CefDragHandler> IWebView::GetDragHandler()
{
    CHECK_REFCOUNTING(nullptr);

    return _drag_handler;
}

CefRefPtr<CefDisplayHandler> IWebView::GetDisplayHandler()
{
    CHECK_REFCOUNTING(nullptr);

    return _display_handler;
}

CefRefPtr<CefLifeSpanHandler> IWebView::GetLifeSpanHandler()
{
    CHECK_REFCOUNTING(nullptr);

    return _life_span_handler;
}

CefRefPtr<CefLoadHandler> IWebView::GetLoadHandler()
{
    CHECK_REFCOUNTING(nullptr);

    return _load_handler;
}

CefRefPtr<CefRenderHandler> IWebView::GetRenderHandler()
{
    CHECK_REFCOUNTING(nullptr);

    return _render_handler;
}

CefRefPtr<CefRequestHandler> IWebView::GetRequestHandler()
{
    CHECK_REFCOUNTING(nullptr);

    return _request_handler;
}

CefRefPtr<CefContextMenuHandler> IWebView::GetContextMenuHandler()
{
    CHECK_REFCOUNTING(nullptr);

    return _context_menu_handler;
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

    if (_render_handler != nullptr)
    {
        _render_handler->Resize(width, height);
        _browser.value()->GetHost()->WasResized();
    }
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
