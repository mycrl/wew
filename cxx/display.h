

#ifndef LIBWEBVIEW_DISPLAY_H
#define LIBWEBVIEW_DISPLAY_H
#pragma once

#include "include/cef_app.h"
#include "webview.h"

class IDisplay : public CefDisplayHandler
{
public:
    IDisplay(CefSettings& cef_settings, PageObserver observer, void* ctx);
    ~IDisplay()
    {
        IClose();
    }
    
    /* CefDisplayHandler */
    
    virtual void OnTitleChange(CefRefPtr<CefBrowser> browser, const CefString& title) override;
    virtual void OnFullscreenModeChange(CefRefPtr<CefBrowser> browser, bool fullscreen) override;
    
    void IClose();
    
private:
    bool _is_closed = false;
    
    CefSettings& _cef_settings;
    PageObserver _observer;
    void* _ctx;
    
    IMPLEMENT_REFCOUNTING(IDisplay);
};

#endif  // LIBWEBVIEW_DISPLAY_H
