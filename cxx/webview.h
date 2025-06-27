//
//  webview.h
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#ifndef webview_h
#define webview_h
#pragma once

#include <float.h>
#include <optional>

#include "include/cef_app.h"

#include "request.h"
#include "util.h"
#include "wew.h"

class IWebViewDrag : public CefDragHandler
{
  public:
    ///
    /// Called when an external drag event enters the browser window.
    ///
    bool OnDragEnter(CefRefPtr<CefBrowser> browser,
                     CefRefPtr<CefDragData> dragData,
                     CefDragHandler::DragOperationsMask mask) override;

  private:
    IMPLEMENT_REFCOUNTING(IWebViewDrag);
};

class IWebViewContextMenu : public CefContextMenuHandler
{
  public:
    ///
    /// Called before a context menu is displayed.
    ///
    void OnBeforeContextMenu(CefRefPtr<CefBrowser> browser,
                             CefRefPtr<CefFrame> frame,
                             CefRefPtr<CefContextMenuParams> params,
                             CefRefPtr<CefMenuModel> model) override;

    ///
    /// Called to execute a command selected from the context menu.
    ///
    /// Return true if the command was handled or false for the default implementation.
    ///
    bool OnContextMenuCommand(CefRefPtr<CefBrowser> browser,
                              CefRefPtr<CefFrame> frame,
                              CefRefPtr<CefContextMenuParams> params,
                              int command_id,
                              EventFlags event_flags) override;

  private:
    IMPLEMENT_REFCOUNTING(IWebViewContextMenu);
};

class IWebViewLoad : public CefLoadHandler
{
  public:
    IWebViewLoad(WebViewHandler &handler);

    ///
    /// Called after a navigation has been committed and before the browser begins
    /// loading contents in the frame.
    ///
    void OnLoadStart(CefRefPtr<CefBrowser> browser, CefRefPtr<CefFrame> frame, TransitionType transition_type) override;

    ///
    /// Called when the browser is done loading a frame.
    ///
    void OnLoadEnd(CefRefPtr<CefBrowser> browser, CefRefPtr<CefFrame> frame, int httpStatusCode) override;

    ///
    /// Called when a navigation fails or is canceled.
    ///
    void OnLoadError(CefRefPtr<CefBrowser> browser,
                     CefRefPtr<CefFrame> frame,
                     ErrorCode error_code,
                     const CefString &error_text,
                     const CefString &failed_url) override;

  private:
    WebViewHandler &_handler;

    IMPLEMENT_REFCOUNTING(IWebViewLoad);
};

class IWebViewLifeSpan : public CefLifeSpanHandler
{
  public:
    IWebViewLifeSpan(std::optional<CefRefPtr<CefBrowser>> &browser, WebViewHandler &handler);

    ///
    /// Called after a new browser is created.
    ///
    void OnAfterCreated(CefRefPtr<CefBrowser> browser) override;

    ///
    /// Called when an Alloy style browser is ready to be closed, meaning that the
    /// close has already been initiated and that JavaScript unload handlers have
    /// already executed or should be ignored.
    ///
    bool DoClose(CefRefPtr<CefBrowser> browser) override;

    ///
    /// Called immediately before the browser object will be destroyed.
    ///
    /// The browser object is no longer valid after this callback returns.
    ///
    void OnBeforeClose(CefRefPtr<CefBrowser> browser) override;

    ///
    /// Called on the UI thread before a new popup browser is created.
    ///
    bool OnBeforePopup(CefRefPtr<CefBrowser> browser,
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
                       bool *no_javascript_access) override;

  private:
    std::optional<CefRefPtr<CefBrowser>> &_browser;
    WebViewHandler &_handler;

    IMPLEMENT_REFCOUNTING(IWebViewLifeSpan);
};

class IWebViewDisplay : public CefDisplayHandler
{
  public:
    IWebViewDisplay(WebViewHandler &handler);

    ///
    /// Called when the page title changes.
    ///
    void OnTitleChange(CefRefPtr<CefBrowser> browser, const CefString &title) override;

    ///
    /// Called when web content in the page has toggled fullscreen mode.
    ///
    void OnFullscreenModeChange(CefRefPtr<CefBrowser> browser, bool fullscreen) override;

  private:
    WebViewHandler &_handler;

    IMPLEMENT_REFCOUNTING(IWebViewDisplay);
};

class IWebViewRender : public CefRenderHandler
{
  public:
    IWebViewRender(const WebViewSettings *settings, WebViewHandler &handler);

    ///
    /// Called to allow the client to fill in the CefScreenInfo object with
    /// appropriate values.
    ///
    bool GetScreenInfo(CefRefPtr<CefBrowser> browser, CefScreenInfo &screen_info) override;

    ///
    /// Called when the IME composition range has changed.
    ///
    void OnImeCompositionRangeChanged(CefRefPtr<CefBrowser> browser,
                                      const CefRange &selected_range,
                                      const RectList &character_bounds) override;

    ///
    /// Called to retrieve the view rectangle in screen DIP coordinates. This
    /// method must always provide a non-empty rectangle.
    ///
    void GetViewRect(CefRefPtr<CefBrowser> browser, CefRect &rect) override;

    ///
    /// Called when an element should be painted. Pixel values passed to this
    /// method are scaled relative to view coordinates based on the value of
    /// CefScreenInfo.device_scale_factor returned from GetScreenInfo.
    ///
    void OnPaint(CefRefPtr<CefBrowser> browser,
                 PaintElementType type,
                 const RectList &dirtyRects,
                 const void *buffer,
                 int width,
                 int height) override;

    ///
    /// Called when the browser wants to move or resize the popup widget.
    ///
    virtual void OnPopupSize(CefRefPtr<CefBrowser> browser, const CefRect &rect) override;

    void Resize(int width, int height);

  private:
    float _device_scale_factor;
    WebViewHandler &_handler;
    CefRect _popup_rect;
    CefRect _view_rect;
    Rect _texture_rect;

    IMPLEMENT_REFCOUNTING(IWebViewRender);
};

class IWebViewRequest : public CefRequestHandler
{
  public:
    IWebViewRequest(const WebViewSettings *settings);

    ///
    /// Called on the browser process IO thread before a resource request is initiated.
    ///
    CefRefPtr<CefResourceRequestHandler> GetResourceRequestHandler(CefRefPtr<CefBrowser> browser,
                                                                   CefRefPtr<CefFrame> frame,
                                                                   CefRefPtr<CefRequest> request,
                                                                   bool is_navigation,
                                                                   bool is_download,
                                                                   const CefString &request_initiator,
                                                                   bool &disable_default_handling) override;

  private:
    CefRefPtr<CefResourceRequestHandler> _handler = nullptr;

    IMPLEMENT_REFCOUNTING(IWebViewRequest);
};

class IWebView : public CefClient
{
  public:
    IWebView(CefSettings &cef_settings, const WebViewSettings *settings, WebViewHandler handler);
    ~IWebView();

    /* CefClient */

    ///
    /// Return the handler for drag events.
    ///
    CefRefPtr<CefDragHandler> GetDragHandler() override;

    ///
    /// Return the handler for context menus.
    ///
    /// If no handler is provided the default implementation will be used.
    ///
    CefRefPtr<CefContextMenuHandler> GetContextMenuHandler() override;

    ///
    /// Return the handler for browser display state events.
    ///
    CefRefPtr<CefDisplayHandler> GetDisplayHandler() override;

    ///
    /// Return the handler for browser life span events.
    ///
    CefRefPtr<CefLifeSpanHandler> GetLifeSpanHandler() override;

    ///
    /// Return the handler for browser load status events.
    ///
    CefRefPtr<CefLoadHandler> GetLoadHandler() override;

    ///
    /// Return the handler for off-screen rendering events.
    ///
    CefRefPtr<CefRenderHandler> GetRenderHandler() override;

    ///
    /// Return the handler for browser request events.
    ///
    CefRefPtr<CefRequestHandler> GetRequestHandler() override;

    ///
    /// Called when a new message is received from a different process.
    ///
    /// Return true if the message was handled or false otherwise.
    ///
    bool OnProcessMessageReceived(CefRefPtr<CefBrowser> browser,
                                  CefRefPtr<CefFrame> frame,
                                  CefProcessId source_process,
                                  CefRefPtr<CefProcessMessage> message) override;

    void Close();
    void SetFocus(bool enable);
    void Resize(int width, int height);
    void SetDevToolsOpenState(bool is_open);
    const void *GetWindowHandle();
    void SendMessage(std::string message);
    void OnKeyboard(cef_key_event_t event);
    void OnMouseClick(cef_mouse_event_t event, cef_mouse_button_type_t button, bool pressed);
    void OnMouseMove(cef_mouse_event_t event);
    void OnMouseWheel(cef_mouse_event_t event, int x, int y);
    void OnTouch(cef_touch_event_t event);
    void OnIMEComposition(std::string input);
    void OnIMESetComposition(std::string input, int x, int y);

  private:
    CefRefPtr<IWebViewDrag> _drag_handler = nullptr;
    CefRefPtr<IWebViewLoad> _load_handler = nullptr;
    CefRefPtr<IWebViewRender> _render_handler = nullptr;
    CefRefPtr<IWebViewRequest> _request_handler = nullptr;
    CefRefPtr<IWebViewDisplay> _display_handler = nullptr;
    CefRefPtr<IWebViewLifeSpan> _life_span_handler = nullptr;
    CefRefPtr<IWebViewContextMenu> _context_menu_handler = nullptr;

    std::optional<CefRefPtr<CefBrowser>> _browser = std::nullopt;
    WebViewHandler _handler;

    IMPLEMENT_RUNNING;
    IMPLEMENT_REFCOUNTING(IWebView);
};

typedef struct
{
    CefRefPtr<IWebView> ref;
} WebView;

#endif /* webview_h */
