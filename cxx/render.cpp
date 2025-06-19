//
//  render.cpp
//  webview
//
//  Created by Mr.Panda on 2023/4/26.
//

#include "render.h"

#include <float.h>

IRender::IRender(const PageOptions* settings, PageObserver observer, void* ctx)
    : _observer(observer)
    , _ctx(ctx)
    , _width(settings->width)
    , _height(settings->height)
    , _device_scale_factor(settings->device_scale_factor)
{
}

void IRender::SetBrowser(CefRefPtr<CefBrowser> browser)
{
    _browser = browser;
}

void IRender::OnImeCompositionRangeChanged(CefRefPtr<CefBrowser> browser,
                                           const CefRange& selected_range,
                                           const RectList& character_bounds)
{
    if (character_bounds.size() == 0)
    {
        return;
    }

    Rect rect;
    rect.x = character_bounds[0].x;
    rect.y = character_bounds[0].y;
    rect.width = character_bounds[0].width;
    rect.height = character_bounds[0].height;
    _observer.on_ime_rect(rect, _ctx);
}

void IRender::GetViewRect(CefRefPtr<CefBrowser> browser, CefRect& rect)
{
    if (is_closed)
    {
        return;
    }

    rect = CefRect(0, 0, _width, _height);
}

void IRender::OnPaint(CefRefPtr<CefBrowser> browser,
                      PaintElementType type,
                      const RectList& dirtyRects,
                      const void* buffer,  // BGRA32
                      int width,
                      int height)
{
    if (is_closed)
    {
        return;
    }

    if (buffer == nullptr)
    {
        return;
    }

    _observer.on_frame(buffer, width, height, _ctx);
}

bool IRender::GetScreenInfo(CefRefPtr<CefBrowser> browser, CefScreenInfo& info)
{
    if (is_closed)
    {
        return false;
    }

    if (abs(info.device_scale_factor - _device_scale_factor) > FLT_EPSILON)
    {
        info.device_scale_factor = _device_scale_factor;
        return true;
    }
    else
    {
        return false;
    }

    return false;
}

void IRender::Resize(int width, int height)
{
    if (is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    _width = width;
    _height = height;
    _browser.value()->GetHost()->WasResized();
}

void IRender::IClose()
{
    _browser = std::nullopt;
    is_closed = true;
}
