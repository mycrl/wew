//
//  browser.h
//  webview
//
//  Created by Mr.Panda on 2023/4/26.
//

#ifndef LIBWEBVIEW_BROWSER_H
#define LIBWEBVIEW_BROWSER_H
#pragma once

#include <optional>

#include "control.h"
#include "display.h"
#include "include/cef_app.h"
#include "render.h"
#include "webview.h"

class IPage : public CefClient,
public CefDragHandler,
public CefContextMenuHandler,
public CefLoadHandler,
public CefLifeSpanHandler,
public IControl,
public IRender,
public IDisplay
{
public:
    IPage(CefSettings& cef_settings,
          const PageOptions* settings,
          PageObserver observer,
          void* ctx);
    
    ~IPage()
    {
        IClose();
    }
    
    /* CefContextMenuHandler */
    
    virtual void OnBeforeContextMenu(CefRefPtr<CefBrowser> browser,
                                     CefRefPtr<CefFrame> frame,
                                     CefRefPtr<CefContextMenuParams> params,
                                     CefRefPtr<CefMenuModel> model) override;
    virtual bool OnContextMenuCommand(CefRefPtr<CefBrowser> browser,
                                      CefRefPtr<CefFrame> frame,
                                      CefRefPtr<CefContextMenuParams> params,
                                      int command_id,
                                      EventFlags event_flags) override;
    
    /* CefClient */
    
    virtual CefRefPtr<CefDragHandler> GetDragHandler() override;
    virtual CefRefPtr<CefContextMenuHandler> GetContextMenuHandler() override;
    virtual CefRefPtr<CefDisplayHandler> GetDisplayHandler() override;
    virtual CefRefPtr<CefLifeSpanHandler> GetLifeSpanHandler() override;
    virtual CefRefPtr<CefLoadHandler> GetLoadHandler() override;
    virtual CefRefPtr<CefRenderHandler> GetRenderHandler() override;
    virtual bool OnProcessMessageReceived(CefRefPtr<CefBrowser> browser,
                                          CefRefPtr<CefFrame> frame,
                                          CefProcessId source_process,
                                          CefRefPtr<CefProcessMessage> message) override;
    
    /* CefLoadHandler */
    
    virtual void OnLoadStart(CefRefPtr<CefBrowser> browser,
                             CefRefPtr<CefFrame> frame,
                             TransitionType transition_type) override;
    virtual void OnLoadEnd(CefRefPtr<CefBrowser> browser,
                           CefRefPtr<CefFrame> frame,
                           int httpStatusCode) override;
    virtual void OnLoadError(CefRefPtr<CefBrowser> browser,
                             CefRefPtr<CefFrame> frame,
                             ErrorCode error_code,
                             const CefString& error_text,
                             const CefString& failed_url) override;
    
    /* CefLifeSpanHandler */
    
    
    virtual void OnAfterCreated(CefRefPtr<CefBrowser> browser) override;
    virtual bool DoClose(CefRefPtr<CefBrowser> browser) override;
    virtual void OnBeforeClose(CefRefPtr<CefBrowser> browser) override;
    virtual bool OnBeforePopup(CefRefPtr<CefBrowser> browser,
                               CefRefPtr<CefFrame> frame,
                               int popup_id,
                               const CefString& target_url,
                               const CefString& target_frame_name,
                               WindowOpenDisposition target_disposition,
                               bool user_gesture,
                               const CefPopupFeatures& popupFeatures,
                               CefWindowInfo& windowInfo,
                               CefRefPtr<CefClient>& client,
                               CefBrowserSettings& settings,
                               CefRefPtr<CefDictionaryValue>& extra_info,
                               bool* no_javascript_access) override;
    
    /* CefDragHandler */
    
    virtual bool OnDragEnter(CefRefPtr<CefBrowser> browser,
                             CefRefPtr<CefDragData> dragData,
                             CefDragHandler::DragOperationsMask mask) override;
    
    void IClose();
    void SetDevToolsOpenState(bool is_open);
    const void* GetWindowHandle();
    void ISendMessage(std::string message);
private:
    bool _is_closed = false;
    std::optional<CefRefPtr<CefBrowser>> _browser = std::nullopt;
    
    CefSettings& _cef_settings;
    PageObserver _observer;
    void* _ctx;
    
    IMPLEMENT_REFCOUNTING(IPage);
};

#endif  // LIBWEBVIEW_BROWSER_H
