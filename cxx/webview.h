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
    const char* scheme_dir_path;
    
    /// The directory where data for the global browser cache will be stored on disk.
    const char* cache_dir_path;
    
    /// The path to a separate executable that will be launched for sub-processes.
    const char* browser_subprocess_path;
    
    /// Set to true (1) to enable windowless (off-screen) rendering support.
    ///
    /// Do not enable this value if the application does not use windowless rendering as it may reduce
    /// rendering performance on some systems.
    bool windowless_rendering_enabled;
    
    /// Set to true (1) to control browser process main (UI) thread message pump scheduling via the
    /// CefBrowserProcessHandler::OnScheduleMessagePumpWork() callback.
    bool external_message_pump;
   
    /// The path to the CEF framework directory on macOS.
    ///
    /// If this value is empty then the framework must exist at
    /// "Contents/Frameworks/Chromium Embedded Framework.framework" in the top-level app bundle.
    /// If this value is non-empty then it must be an absolute path. Also configurable using the
    /// "framework-dir-path" command-line switch.
    const char* framework_dir_path;
    
    /// The path to the main bundle on macOS.
    const char* main_bundle_path;
    
    /// Set to true (1) to have the browser process message loop run in a separate thread.
    bool multi_threaded_message_loop;
} AppOptions;

typedef struct
{
    /// window size width.
    uint32_t width;
    
    /// window size height.
    uint32_t height;
    
    /// window device scale factor.
    float device_scale_factor;
    
    /// page defalt fixed font size.
    int default_fixed_font_size;
    
    /// page defalt font size.
    int default_font_size;
    
    /// Controls whether JavaScript can be executed.
    bool javascript;
    
    /// Controls whether JavaScript can access the clipboard.
    bool javascript_access_clipboard;
    
    /// Controls whether local storage can be used.
    bool local_storage;
    
    /// The maximum rate in frames per second (fps) that CefRenderHandler::OnPaint will be called for a
    /// windowless browser.
    uint32_t windowless_frame_rate;
    
    /// External native window handle.
    const void* window_handle;
} PageOptions;

typedef enum
{
    kLeft = 0,
    kRight = 1,
    kMiddle = 2,
} MouseButtons;

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
    BeforeLoad = 1,
    Load = 2,
    LoadError = 3,
    RequestClose = 4,
    Close = 5,
} PageState;

typedef struct
{
    int x;
    int y;
    int width;
    int height;
} Rect;

typedef struct
{
    void (*on_state_change)(PageState state, void* ctx);
    void (*on_ime_rect)(Rect rect, void* ctx);
    void (*on_frame)(const void* buf, int width, int height, void* ctx);
    void (*on_title_change)(const char* title, void* ctx);
    void (*on_fullscreen_change)(bool fullscreen, void* ctx);
    void (*on_message)(const char* message, void* ctx);
} PageObserver;

typedef struct
{
    void (*on_context_initialized)(void* ctx);
    void (*on_schedule_message_pump_work)(int64_t delay_ms, void* ctx);
} AppObserver;

#ifdef __cplusplus
extern "C" {

#endif

EXPORT void run_message_loop();

EXPORT void quit_message_loop();

EXPORT void poll_message_loop();

EXPORT int execute_subprocess(int argc, const char** argv);

EXPORT void* create_app(const AppOptions* settings, AppObserver observer, void* ctx);

EXPORT void execute_app(void* app, int argc, const char** argv);

//
// This function should be called on the main application thread to shut down
// the CEF browser process before the application exits.
//
EXPORT void close_app(void* app);

EXPORT void* create_page(void* app,
                         const char* url,
                         const PageOptions* settings,
                         PageObserver observer,
                         void* ctx);

EXPORT void close_page(void* page);

//
// Send a mouse click event to the browser.
//
EXPORT void page_send_mouse_click(void* page,
                                  MouseButtons button,
                                  bool pressed);

//
// Send a mouse click event to the browser. The |x| and |y| coordinates are
// relative to the upper-left corner of the view.
//
EXPORT void page_send_mouse_click_with_pos(void* page,
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
EXPORT void page_send_mouse_wheel(void* page, int x, int y);

//
// Send a mouse move event to the browser. The |x| and |y| coordinates are
// relative to the upper-left corner of the view.
//
EXPORT void page_send_mouse_move(void* page, int x, int y);

//
// Send a key event to the browser.
//
EXPORT void page_send_keyboard(void* page,
                               int scan_code,
                               bool pressed,
                               Modifiers modifiers);
//
// Send a touch event to the browser.
//
EXPORT void page_send_touch(void* page,
                            int id,
                            int x,
                            int y,
                            TouchEventType type,
                            TouchPointerType pointer_type);

EXPORT void page_send_message(void* page, const char* message);

EXPORT void page_set_devtools_state(void* page, bool is_open);

EXPORT void page_resize(void* page, int width, int height);

EXPORT const void* page_get_hwnd(void* page);

EXPORT void page_send_ime_composition(void* page, const char* input);

EXPORT void page_send_ime_set_composition(void* page, const char* input, int x, int y);

#ifdef __cplusplus
}
#endif

#endif  // LIBWEBVIEW_WEBVIEW_H
