//
//  browser.cpp
//  webview
//
//  Created by Mr.Panda on 2023/4/26.
//

#include "page.h"

#include "include/base/cef_callback.h"
#include "include/cef_app.h"
#include "include/wrapper/cef_closure_task.h"
#include "include/wrapper/cef_helpers.h"

IPage::IPage(CefSettings& cef_settings,
             const PageOptions* settings,
             PageObserver observer,
             void* ctx)
:_observer(observer)
, _ctx(ctx)
, _cef_settings(cef_settings)
, IRender(settings, observer, ctx)
, IDisplay(cef_settings, observer, ctx)
{
}

CefRefPtr<CefDragHandler> IPage::GetDragHandler()
{
    return this;
}

void IPage::OnBeforeContextMenu(CefRefPtr<CefBrowser> browser,
                                CefRefPtr<CefFrame> frame,
                                CefRefPtr<CefContextMenuParams> params,
                                CefRefPtr<CefMenuModel> model)
{
    CEF_REQUIRE_UI_THREAD();
    
    if (params->GetTypeFlags() & (CM_TYPEFLAG_SELECTION | CM_TYPEFLAG_EDITABLE))
    {
        return;
    }
    
    model->Clear();
}

CefRefPtr<CefContextMenuHandler> IPage::GetContextMenuHandler()
{
    return this;
}

bool IPage::OnContextMenuCommand(CefRefPtr<CefBrowser> browser,
                                 CefRefPtr<CefFrame> frame,
                                 CefRefPtr<CefContextMenuParams> params,
                                 int command_id,
                                 EventFlags event_flags)
{
    CEF_REQUIRE_UI_THREAD();
    return false;
};

CefRefPtr<CefDisplayHandler> IPage::GetDisplayHandler()
{
    if (_is_closed)
    {
        return nullptr;
    }
    
    return this;
}

CefRefPtr<CefLifeSpanHandler> IPage::GetLifeSpanHandler()
{
    if (_is_closed)
    {
        return nullptr;
    }
    
    return this;
}

CefRefPtr<CefLoadHandler> IPage::GetLoadHandler()
{
    if (_is_closed)
    {
        return nullptr;
    }
    
    return this;
}

CefRefPtr<CefRenderHandler> IPage::GetRenderHandler()
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


void IPage::OnLoadStart(CefRefPtr<CefBrowser> browser,
                        CefRefPtr<CefFrame> frame,
                        TransitionType transition_type)
{
    if (_is_closed)
    {
        return;
    }
    
    _observer.on_state_change(PageState::BeforeLoad, _ctx);
}

void IPage::OnLoadEnd(CefRefPtr<CefBrowser> browser,
                      CefRefPtr<CefFrame> frame,
                      int httpStatusCode)
{
    CEF_REQUIRE_UI_THREAD();
    
    if (_is_closed)
    {
        return;
    }
    
    _observer.on_state_change(PageState::Load, _ctx);
}

void IPage::OnLoadError(CefRefPtr<CefBrowser> browser,
                        CefRefPtr<CefFrame> frame,
                        ErrorCode error_code,
                        const CefString& error_text,
                        const CefString& failed_url)
{
    CEF_REQUIRE_UI_THREAD();
    
    if (_is_closed)
    {
        return;
    }
    
    _observer.on_state_change(PageState::LoadError, _ctx);
    
    if (error_code == ERR_ABORTED)
    {
        return;
    }
}

void IPage::OnAfterCreated(CefRefPtr<CefBrowser> browser)
{
    if (_is_closed)
    {
        return;
    }
    
    browser->GetHost()->WasResized();
    
    IRender::SetBrowser(browser);
    IControl::SetBrowser(browser);
    _browser = browser;
}

bool IPage::DoClose(CefRefPtr<CefBrowser> browser)
{
    CEF_REQUIRE_UI_THREAD();
    
    _observer.on_state_change(PageState::RequestClose, _ctx);
    
    return false;
}

bool IPage::OnBeforePopup(CefRefPtr<CefBrowser> browser,
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
                          bool* no_javascript_access)
{
    browser->GetMainFrame()->LoadURL(target_url);

    return true;
}

bool IPage::OnDragEnter(CefRefPtr<CefBrowser> browser,
                        CefRefPtr<CefDragData> dragData,
                        CefDragHandler::DragOperationsMask mask)
{
    return true;
}

void IPage::OnBeforeClose(CefRefPtr<CefBrowser> browser)
{
    CEF_REQUIRE_UI_THREAD();
    
    _observer.on_state_change(PageState::Close, _ctx);
    _browser = std::nullopt;
}

void IPage::SetDevToolsOpenState(bool is_open)
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
        _browser.value()->GetHost()->ShowDevTools(CefWindowInfo(),
                                                  nullptr,
                                                  CefBrowserSettings(),
                                                  CefPoint());
    }
    else
    {
        _browser.value()->GetHost()->CloseDevTools();
    }
}

const void* IPage::GetWindowHandle()
{
    return _browser.has_value() ? _browser.value()->GetHost()->GetWindowHandle() : nullptr;
}

bool IPage::OnProcessMessageReceived(CefRefPtr<CefBrowser> browser,
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
    _observer.on_message(payload.c_str(), _ctx);
    return true;
}

void IPage::ISendMessage(std::string message)
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

void IPage::IClose()
{
    if (_is_closed)
    {
        return;
    }
    
    if (!_browser.has_value())
    {
        return;
    }
    
    IRender::IClose();
    IDisplay::IClose();
    IControl::IClose();
    _browser.value()->GetHost()->CloseBrowser(true);
    
    _browser = std::nullopt;
    _is_closed = true;
}
