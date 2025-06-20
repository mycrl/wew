//
//  lib.h
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#ifndef library_h
#define library_h
#pragma once

#ifdef WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif

#include "include/internal/cef_types.h"

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct
{
    /// The directory where data for the global browser cache will be stored on disk.
    const char *cache_dir_path;

    /// The path to a separate executable that will be launched for sub-processes.
    const char *browser_subprocess_path;

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
    const char *framework_dir_path;

    /// The path to the main bundle on macOS.
    const char *main_bundle_path;

    /// Set to true (1) to have the browser process message loop run in a separate thread.
    bool multi_threaded_message_loop;
} RuntimeSettings;

typedef struct
{
    void (*on_context_initialized)(void *context);
    void (*on_schedule_message_pump_work)(int64_t delay_ms, void *context);
    void *context;
} RuntimeHandler;

typedef struct
{
    /// window size width.
    uint32_t width;

    /// window size height.
    uint32_t height;

    /// window device scale factor.
    float device_scale_factor;

    /// webview defalt fixed font size.
    int default_fixed_font_size;

    /// webview defalt font size.
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
    const void *window_handle;
} WebViewSettings;

typedef enum
{
    BeforeLoad = 1,
    Loaded = 2,
    LoadError = 3,
    RequestClose = 4,
    Close = 5,
} WebViewState;

typedef cef_mouse_event_t MouseEvent;

typedef cef_touch_event_t TouchEvent;

typedef cef_key_event_t KeyEvent;

typedef struct
{
    void (*on_state_change)(WebViewState state, void *context);
    void (*on_ime_rect)(cef_rect_t rect, void *context);
    void (*on_frame)(const void *buf, int width, int height, void *context);
    void (*on_title_change)(const char *title, void *context);
    void (*on_fullscreen_change)(bool fullscreen, void *context);
    void (*on_message)(const char *message, void *context);
    void *context;
} WebViewHandler;

typedef struct
{
    const char *url;
    const char *method;
    const char *referrer;
} ResourceRequest;

typedef struct
{
    int status_code;
    uint64_t content_length;
    const char *mime_type;
} ResourceResponse;

typedef struct
{
    bool (*open)(void *context);
    bool (*skip)(size_t size, size_t *cursor, void *context);
    bool (*read)(void *buffer, size_t size, size_t *cursor, void *context);
    void (*get_response)(ResourceResponse *response, void *context);
    void (*cancel)(void *context);
    void (*destroy)(void *context);
    void *context;
} ResourceHandler;

typedef struct
{
    ResourceHandler *(*create_resource_handler)(ResourceRequest *request, void *context);
    void (*destroy_resource_handler)(ResourceHandler *handler);
    void *context;
} ResourceRequestHandler;

#ifdef __cplusplus
extern "C"
{

#endif

    EXPORT int execute_subprocess(int argc, const char **argv);

    EXPORT void run_message_loop();

    EXPORT void quit_message_loop();

    EXPORT void poll_message_loop();

    EXPORT void *create_runtime(const RuntimeSettings *settings, RuntimeHandler handler);

    EXPORT void execute_runtime(void *runtime_ptr, int argc, const char **argv);

    //
    // This function should be called on the main application thread to shut down
    // the CEF browser process before the application exits.
    //
    EXPORT void close_runtime(void *runtime_ptr);

    EXPORT void *create_webview(void *runtime_ptr,
                                const char *url,
                                const WebViewSettings *settings,
                                WebViewHandler handler);

    EXPORT void close_webview(void *webview_ptr);

    //
    // Send a mouse click event to the browser.
    //
    EXPORT void webview_mouse_click(void *webview_ptr, cef_mouse_event_t event, cef_mouse_button_type_t button, bool pressed);

    //
    // Send a mouse wheel event to the browser. The |x| and |y| coordinates are
    // relative to the upper-left corner of the view. The |deltaX| and |deltaY|
    // values represent the movement delta in the X and Y directions
    // respectively. In order to scroll inside select popups with window
    // rendering disabled CefRenderHandler::GetScreenPoint should be implemented
    // properly.
    //
    EXPORT void webview_mouse_wheel(void *webview_ptr, cef_mouse_event_t event, int x, int y);

    //
    // Send a mouse move event to the browser. The |x| and |y| coordinates are
    // relative to the upper-left corner of the view.
    //
    EXPORT void webview_mouse_move(void *webview_ptr, cef_mouse_event_t event);

    //
    // Send a key event to the browser.
    //
    EXPORT void webview_keyboard(void *webview_ptr, cef_key_event_t event);

    //
    // Send a touch event to the browser.
    //
    EXPORT void webview_touch(void *webview_ptr, cef_touch_event_t event);

    EXPORT void webview_ime_composition(void *webview_ptr, const char *input);

    EXPORT void webview_ime_set_composition(void *webview_ptr, const char *input, int x, int y);

    EXPORT void webview_send_message(void *webview_ptr, const char *message);

    EXPORT void webview_set_devtools_state(void *webview_ptr, bool is_open);

    EXPORT void webview_resize(void *webview_ptr, int width, int height);

    EXPORT const void *webview_get_window_handle(void *webview_ptr);

    EXPORT void webview_set_request_handler(void *webview_ptr, ResourceRequestHandler *handler);

#ifdef __cplusplus
}
#endif

#endif /* library_h */
