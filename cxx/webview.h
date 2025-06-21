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

#include "library.h"

class IWebView : public CefClient,
                 public CefDragHandler,
                 public CefContextMenuHandler,
                 public CefLoadHandler,
                 public CefLifeSpanHandler,
                 public CefDisplayHandler,
                 public CefRenderHandler
{
  public:
    IWebView(CefSettings &cef_settings, const WebViewSettings *settings, WebViewHandler handler);
    ~IWebView();

    /* CefContextMenuHandler */

    ///
    /// Called before a context menu is displayed. |params| provides information
    /// about the context menu state. |model| initially contains the default
    /// context menu. The |model| can be cleared to show no context menu or
    /// modified to show a custom menu. Do not keep references to |params| or
    /// |model| outside of this callback.
    ///
    virtual void OnBeforeContextMenu(CefRefPtr<CefBrowser> browser,
                                     CefRefPtr<CefFrame> frame,
                                     CefRefPtr<CefContextMenuParams> params,
                                     CefRefPtr<CefMenuModel> model) override;

    ///
    /// Called to execute a command selected from the context menu. Return true if
    /// the command was handled or false for the default implementation. See
    /// cef_menu_id_t for the command ids that have default implementations. All
    /// user-defined command ids should be between MENU_ID_USER_FIRST and
    /// MENU_ID_USER_LAST. |params| will have the same values as what was passed
    /// to OnBeforeContextMenu(). Do not keep a reference to |params| outside of
    /// this callback.
    ///
    virtual bool OnContextMenuCommand(CefRefPtr<CefBrowser> browser,
                                      CefRefPtr<CefFrame> frame,
                                      CefRefPtr<CefContextMenuParams> params,
                                      int command_id,
                                      EventFlags event_flags) override;

    /* CefClient */

    ///
    /// Return the handler for drag events.
    ///
    virtual CefRefPtr<CefDragHandler> GetDragHandler() override;

    ///
    /// Return the handler for context menus. If no handler is provided the
    /// default implementation will be used.
    ///
    virtual CefRefPtr<CefContextMenuHandler> GetContextMenuHandler() override;

    ///
    /// Return the handler for browser display state events.
    ///
    virtual CefRefPtr<CefDisplayHandler> GetDisplayHandler() override;

    ///
    /// Return the handler for browser life span events.
    ///
    virtual CefRefPtr<CefLifeSpanHandler> GetLifeSpanHandler() override;

    ///
    /// Return the handler for browser load status events.
    ///
    virtual CefRefPtr<CefLoadHandler> GetLoadHandler() override;

    ///
    /// Return the handler for off-screen rendering events.
    ///
    virtual CefRefPtr<CefRenderHandler> GetRenderHandler() override;

    ///
    /// Called when a new message is received from a different process. Return
    /// true if the message was handled or false otherwise.  It is safe to keep a
    /// reference to |message| outside of this callback.
    ///
    virtual bool OnProcessMessageReceived(CefRefPtr<CefBrowser> browser,
                                          CefRefPtr<CefFrame> frame,
                                          CefProcessId source_process,
                                          CefRefPtr<CefProcessMessage> message) override;

    /* CefLoadHandler */

    ///
    /// Called after a navigation has been committed and before the browser begins
    /// loading contents in the frame. The |frame| value will never be empty --
    /// call the IsMain() method to check if this frame is the main frame.
    /// |transition_type| provides information about the source of the navigation
    /// and an accurate value is only available in the browser process. Multiple
    /// frames may be loading at the same time. Sub-frames may start or continue
    /// loading after the main frame load has ended. This method will not be
    /// called for same page navigations (fragments, history state, etc.) or for
    /// navigations that fail or are canceled before commit. For notification of
    /// overall browser load status use OnLoadingStateChange instead.
    ///
    virtual void OnLoadStart(CefRefPtr<CefBrowser> browser,
                             CefRefPtr<CefFrame> frame,
                             TransitionType transition_type) override;

    ///
    /// Called when the browser is done loading a frame. The |frame| value will
    /// never be empty -- call the IsMain() method to check if this frame is the
    /// main frame. Multiple frames may be loading at the same time. Sub-frames
    /// may start or continue loading after the main frame load has ended. This
    /// method will not be called for same page navigations (fragments, history
    /// state, etc.) or for navigations that fail or are canceled before commit.
    /// For notification of overall browser load status use OnLoadingStateChange
    /// instead.
    ///
    virtual void OnLoadEnd(CefRefPtr<CefBrowser> browser, CefRefPtr<CefFrame> frame, int httpStatusCode) override;

    ///
    /// Called when a navigation fails or is canceled. This method may be called
    /// by itself if before commit or in combination with OnLoadStart/OnLoadEnd if
    /// after commit. |errorCode| is the error code number, |errorText| is the
    /// error text and |failedUrl| is the URL that failed to load.
    /// See net\base\net_error_list.h for complete descriptions of the error
    /// codes.
    ///
    virtual void OnLoadError(CefRefPtr<CefBrowser> browser,
                             CefRefPtr<CefFrame> frame,
                             ErrorCode error_code,
                             const CefString &error_text,
                             const CefString &failed_url) override;

    /* CefLifeSpanHandler */

    ///
    /// Called after a new browser is created. It is now safe to begin performing
    /// actions with |browser|. CefFrameHandler callbacks related to initial main
    /// frame creation will arrive before this callback. See CefFrameHandler
    /// documentation for additional usage information.
    ///
    virtual void OnAfterCreated(CefRefPtr<CefBrowser> browser) override;

    ///
    /// Called when an Alloy style browser is ready to be closed, meaning that the
    /// close has already been initiated and that JavaScript unload handlers have
    /// already executed or should be ignored. This may result directly from a
    /// call to CefBrowserHost::[Try]CloseBrowser() or indirectly if the browser's
    /// top-level parent window was created by CEF and the user attempts to
    /// close that window (by clicking the 'X', for example). DoClose() will not
    /// be called if the browser's host window/view has already been destroyed
    /// (via parent window/view hierarchy tear-down, for example), as it is no
    /// longer possible to customize the close behavior at that point.
    ///
    virtual bool DoClose(CefRefPtr<CefBrowser> browser) override;

    ///
    /// Called immediately before the browser object will be destroyed. The
    /// browser object is no longer valid after this callback returns.
    ///
    virtual void OnBeforeClose(CefRefPtr<CefBrowser> browser) override;

    ///
    /// Called on the UI thread before a new popup browser is created. The
    /// |browser| and |frame| values represent the source of the popup request
    /// (opener browser and frame). The |popup_id| value uniquely identifies the
    /// popup in the context of the opener browser. The |target_url| and
    /// |target_frame_name| values indicate where the popup browser should
    /// navigate and may be empty if not specified with the request. The
    /// |target_disposition| value indicates where the user intended to open the
    /// popup (e.g. current tab, new tab, etc). The |user_gesture| value will be
    /// true if the popup was opened via explicit user gesture (e.g. clicking a
    /// link) or false if the popup opened automatically (e.g. via the
    /// DomContentLoaded event). The |popupFeatures| structure contains additional
    /// information about the requested popup window. To allow creation of the
    /// popup browser optionally modify |windowInfo|, |client|, |settings| and
    /// |no_javascript_access| and return false. To cancel creation of the popup
    /// browser return true. The |client| and |settings| values will default to
    /// the source browser's values. If the |no_javascript_access| value is set to
    /// false the new browser will not be scriptable and may not be hosted in the
    /// same renderer process as the source browser. Any modifications to
    /// |windowInfo| will be ignored if the parent browser is wrapped in a
    /// CefBrowserView. The |extra_info| parameter provides an opportunity to
    /// specify extra information specific to the created popup browser that will
    /// be passed to CefRenderProcessHandler::OnBrowserCreated() in the render
    /// process.
    ///
    virtual bool OnBeforePopup(CefRefPtr<CefBrowser> browser,
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

    /* CefDragHandler */

    ///
    /// Called when an external drag event enters the browser window. |dragData|
    /// contains the drag event data and |mask| represents the type of drag
    /// operation. Return false for default drag handling behavior or true to
    /// cancel the drag event.
    ///
    virtual bool OnDragEnter(CefRefPtr<CefBrowser> browser,
                             CefRefPtr<CefDragData> dragData,
                             CefDragHandler::DragOperationsMask mask) override;

    /* CefDisplayHandler */

    ///
    /// Called when the page title changes.
    ///
    virtual void OnTitleChange(CefRefPtr<CefBrowser> browser, const CefString &title) override;

    ///
    /// Called when web content in the page has toggled fullscreen mode. If
    /// |fullscreen| is true the content will automatically be sized to fill the
    /// browser content area. If |fullscreen| is false the content will
    /// automatically return to its original size and position. With Alloy style
    /// the client is responsible for triggering the fullscreen transition (for
    /// example, by calling CefWindow::SetFullscreen when using Views). With
    /// Chrome style the fullscreen transition will be triggered automatically.
    /// The CefWindowDelegate::OnWindowFullscreenTransition method will be called
    /// during the fullscreen transition for notification purposes.
    ///
    virtual void OnFullscreenModeChange(CefRefPtr<CefBrowser> browser, bool fullscreen) override;

    /* CefRenderHandler */

    ///
    /// Called to allow the client to fill in the CefScreenInfo object with
    /// appropriate values. Return true if the |screen_info| structure has been
    /// modified.
    ///
    /// If the screen info rectangle is left empty the rectangle from GetViewRect
    /// will be used. If the rectangle is still empty or invalid popups may not be
    /// drawn correctly.
    ///
    virtual bool GetScreenInfo(CefRefPtr<CefBrowser> browser, CefScreenInfo &screen_info) override;

    ///
    /// Called when the IME composition range has changed. |selected_range| is the
    /// range of characters that have been selected. |character_bounds| is the
    /// bounds of each character in view coordinates.
    ///
    virtual void OnImeCompositionRangeChanged(CefRefPtr<CefBrowser> browser,
                                              const CefRange &selected_range,
                                              const RectList &character_bounds) override;

    ///
    /// Called to retrieve the view rectangle in screen DIP coordinates. This
    /// method must always provide a non-empty rectangle.
    ///
    virtual void GetViewRect(CefRefPtr<CefBrowser> browser, CefRect &rect) override;

    ///
    /// Called when an element should be painted. Pixel values passed to this
    /// method are scaled relative to view coordinates based on the value of
    /// CefScreenInfo.device_scale_factor returned from GetScreenInfo. |type|
    /// indicates whether the element is the view or the popup widget. |buffer|
    /// contains the pixel data for the whole image. |dirtyRects| contains the set
    /// of rectangles in pixel coordinates that need to be repainted. |buffer|
    /// will be |width|*|height|*4 bytes in size and represents a BGRA image with
    /// an upper-left origin. This method is only called when
    /// CefWindowInfo::shared_texture_enabled is set to false.
    ///
    virtual void OnPaint(CefRefPtr<CefBrowser> browser,
                         PaintElementType type,
                         const RectList &dirtyRects,
                         const void *buffer,
                         int width,
                         int height) override;

    void Close();
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
    std::optional<CefRefPtr<CefBrowser>> _browser;
    bool _is_closed = false;

    cef_rect_t _view_rect;
    float _device_scale_factor;
    CefSettings &_cef_settings;
    WebViewHandler _handler;

    IMPLEMENT_REFCOUNTING(IWebView);
};

typedef struct
{
    CefRefPtr<IWebView> ref;
} WebView;

#endif /* webview_h */
