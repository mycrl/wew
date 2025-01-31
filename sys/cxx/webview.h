//
//  webview.h
//  webview
//
//  Created by Mr.Panda on 2023/4/26.
//

#ifndef LIBWEBVIEW_WEBVIEW_H
#define LIBWEBVIEW_WEBVIEW_H
#pragma once

#ifdef WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif

#include <stdint.h>
#include <stdbool.h>

typedef struct
{
    char* cache_path;
    char* browser_subprocess_path;
    char* scheme_path;
} WebviewOptions;

typedef struct
{
    void* window_handle;
    uint32_t frame_rate;
    uint32_t width;
    uint32_t height;
    float device_scale_factor;
    bool is_offscreen;
} PageOptions;

typedef enum
{
    kLeft,
    kRight,
    kMiddle,
} MouseButtons;

typedef struct
{
    char* success;
    char* failure;
} Result;

typedef enum
{
    kNone = 0,
    kShift = 1,
    kCtrl = 2,
    kAlt = 3,
    kWin = 4,
} Modifiers;

typedef enum
{
    kTouchReleased = 0,
    kTouchPressed = 1,
    kTouchMoved = 2,
    kTouchCancelled = 3,
} TouchEventType;

typedef enum
{
    kTouch = 0,
    kMouse = 1,
    kPen = 2,
    kEraser = 3,
    kUnknown = 4,
} TouchPointerType;

typedef enum
{
    Load = 1,
    LoadError = 2,
    BeforeLoad = 3,
    BeforeClose = 4,
    Close = 5,
} PageState;

typedef struct
{
    int x;
    int y;
    int width;
    int height;
} Rect;

typedef void (*CreateWebviewCallback)(void* ctx);
typedef void (*BridgeOnCallback)(void* cb_ctx, Result ret);
typedef void (*BridgeOnHandler)(const char* req, void* ctx, void* cb_ctx, BridgeOnCallback cb);
typedef void (*BridgeCallCallback)(const char* res, void* ctx);

typedef struct
{
    void (*on_state_change)(PageState state, void* ctx);
    void (*on_ime_rect)(Rect rect, void* ctx);
    void (*on_frame)(const void* buf, int width, int height, void* ctx);
    void (*on_title_change)(const char* title, void* ctx);
    void (*on_fullscreen_change)(bool fullscreen, void* ctx);
    void (*on_bridge)(const char* req, void* ctx, void* cb_ctx, BridgeOnCallback cb);
} PageObserver;

#ifdef __cplusplus
extern "C" {
#endif

    EXPORT void execute_sub_process(int argc, char** argv);

    EXPORT void* create_webview(WebviewOptions* settings, CreateWebviewCallback callback, void* ctx);

    //
    // Run the CEF message loop. Use this function instead of an
    // application-provided message loop to get the best balance between performance
    // and CPU usage. This function will block until a quit message is received by
    // the system.
    //
    EXPORT int webview_run(void* app, int argc, char** argv);

    //
    // This function should be called on the main application thread to shut down
    // the CEF browser process before the application exits.
    //
    EXPORT void webview_exit(void* app);

    EXPORT void* create_page(void* app,
                             const char* url,
                             PageOptions* settings,
                             PageObserver observer,
                             void* ctx);

    EXPORT void page_exit(void* browser);

    //
    // Send a mouse click event to the browser.
    //
    EXPORT void page_send_mouse_click(void* browser,
                                      MouseButtons button,
                                      bool pressed);

    //
    // Send a mouse click event to the browser. The |x| and |y| coordinates are
    // relative to the upper-left corner of the view.
    //
    EXPORT void page_send_mouse_click_with_pos(void* browser,
                                               MouseButtons button,
                                               bool pressed,
                                               int x,
                                               int y);

    //
    // Send a mouse wheel event to the browser. The |x| and |y| coordinates are
    // relative to the upper-left corner of the view. The |deltaX| and |deltaY|
    // values represent the movement delta in the X and Y directions
    // respectively. In order to scroll inside select popups with window
    // rendering disabled CefRenderHandler::GetScreenPoint should be implemented
    // properly.
    //
    EXPORT void page_send_mouse_wheel(void* browser, int x, int y);

    //
    // Send a mouse move event to the browser. The |x| and |y| coordinates are
    // relative to the upper-left corner of the view.
    //
    EXPORT void page_send_mouse_move(void* browser, int x, int y);

    //
    // Send a key event to the browser.
    //
    EXPORT void page_send_keyboard(void* browser,
                                   int scan_code,
                                   bool pressed,
                                   Modifiers modifiers);
    //
    // Send a touch event to the browser.
    //
    EXPORT void page_send_touch(void* browser,
                                int id,
                                int x,
                                int y,
                                TouchEventType type,
                                TouchPointerType pointer_type);

    EXPORT void page_bridge_call(void* browser,
                                 char* req,
                                 BridgeCallCallback callback,
                                 void* ctx);

    EXPORT void page_set_devtools_state(void* browser, bool is_open);

    EXPORT void page_resize(void* browser, int width, int height);

    EXPORT const void* page_get_hwnd(void* browser);

    EXPORT void page_send_ime_composition(void* browser, char* input);

    EXPORT void page_send_ime_set_composition(void* browser, char* input, int x, int y);

#ifdef __cplusplus
}
#endif

#endif  // LIBWEBVIEW_WEBVIEW_H
