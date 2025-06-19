#ifndef LIBWEBVIEW_CONTROL_H
#define LIBWEBVIEW_CONTROL_H
#pragma once

#include <array>
#include <optional>

#include "include/cef_app.h"
#include "webview.h"

#ifdef CEF_X11
#include <X11/XKBlib.h>
#endif

class IMEControl
{
public:
    ~IMEControl()
    {
        IClose();
    }

    void SetBrowser(CefRefPtr<CefBrowser> browser);
    void OnIMEComposition(std::string input);
    void OnIMESetComposition(std::string input, int x, int y);
    void IClose();

private:
    std::optional<CefRefPtr<CefBrowser>> _browser = std::nullopt;
    bool is_closed = false;
};

class IControl : public IMEControl
{
public:
    ~IControl()
    {
        IClose();
    }

    void SetBrowser(CefRefPtr<CefBrowser> browser);
    void OnKeyboard(int scan_code, bool pressed, Modifiers modifiers);
    void OnMouseClick(MouseButtons button, bool pressed);
    void OnMouseClickWithPosition(MouseButtons button, int x, int y, bool pressed);
    void OnMouseMove(int x, int y);
    void OnMouseWheel(int x, int y);
    void OnTouch(int id, int x, int y, cef_touch_event_type_t type, cef_pointer_type_t pointer_type);
    void IClose();

private:
    std::optional<CefRefPtr<CefBrowser>> _browser = std::nullopt;

#ifdef CEF_X11
    Display* _display = XOpenDisplay(nullptr);
#endif
    CefMouseEvent _mouse_event;
    bool _is_closed = false;
};

#endif  // LIBWEBVIEW_CONTROL_H
