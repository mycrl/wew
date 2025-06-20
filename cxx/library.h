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

typedef struct
{
    int x;
    int y;
    int width;
    int height;
} Rect;

///
/// Mouse button types.
///
typedef enum
{
    Left = 0,
    Middle,
    Right,
} MouseButtonType;

typedef enum
{
    None = 0,
    CapsLockOn = 1 << 0,
    ShiftDown = 1 << 1,
    ControlDown = 1 << 2,
    AltDown = 1 << 3,
    LeftMouseButton = 1 << 4,
    MiddleMouseButton = 1 << 5,
    RightMouseButton = 1 << 6,
    CommandDown = 1 << 7,
    NumLockOn = 1 << 8,
    IsKeyPad = 1 << 9,
    IsLeft = 1 << 10,
    IsRight = 1 << 11,
    AltGrDown = 1 << 12,
    IsRepeat = 1 << 13,
    PrecisionScrollingDelta = 1 << 14,
    ScrollByPage = 1 << 15,
} EventFlags;

///
/// Structure representing mouse event information.
///
typedef struct
{
    ///
    /// X coordinate relative to the left side of the view.
    ///
    int x;

    ///
    /// Y coordinate relative to the top side of the view.
    ///
    int y;

    ///
    /// Bit flags describing any pressed modifier keys. See
    /// cef_event_flags_t for values.
    ///
    uint32_t modifiers;
} MouseEvent;

///
/// Touch points states types.
///
typedef enum
{
    Released = 0,
    Pressed,
    Moved,
    Cancelled
} TouchEventType;

///
/// The device type that caused the event.
///
typedef enum
{
    Touch = 0,
    Mouse,
    Pen,
    Eraser,
    Unknown
} PointerType;

///
/// Structure representing touch event information.
///
typedef struct
{
    ///
    /// Id of a touch point. Must be unique per touch, can be any number except
    /// -1. Note that a maximum of 16 concurrent touches will be tracked; touches
    /// beyond that will be ignored.
    ///
    int id;

    ///
    /// X coordinate relative to the left side of the view.
    ///
    float x;

    ///
    /// Y coordinate relative to the top side of the view.
    ///
    float y;

    ///
    /// X radius in pixels. Set to 0 if not applicable.
    ///
    float radius_x;

    ///
    /// Y radius in pixels. Set to 0 if not applicable.
    ///
    float radius_y;

    ///
    /// Rotation angle in radians. Set to 0 if not applicable.
    ///
    float rotation_angle;

    ///
    /// The normalized pressure of the pointer input in the range of [0,1].
    /// Set to 0 if not applicable.
    ///
    float pressure;

    ///
    /// The state of the touch point. Touches begin with one CEF_TET_PRESSED event
    /// followed by zero or more CEF_TET_MOVED events and finally one
    /// CEF_TET_RELEASED or CEF_TET_CANCELLED event. Events not respecting this
    /// order will be ignored.
    ///
    TouchEventType type;

    ///
    /// Bit flags describing any pressed modifier keys. See
    /// cef_event_flags_t for values.
    ///
    uint32_t modifiers;

    ///
    /// The device type that caused the event.
    ///
    PointerType pointer_type;

} TouchEvent;

///
/// Key event types.
///
typedef enum
{
    ///
    /// Notification that a key transitioned from "up" to "down".
    ///
    RawKeyDown = 0,

    ///
    /// Notification that a key was pressed. This does not necessarily correspond
    /// to a character depending on the key and language. Use KEYEVENT_CHAR for
    /// character input.
    ///
    KeyDown,

    ///
    /// Notification that a key was released.
    ///
    KeyUp,

    ///
    /// Notification that a character was typed. Use this for text input. Key
    /// down events may generate 0, 1, or more than one character event depending
    /// on the key, locale, and operating system.
    ///
    Char
} KeyEventType;

///
/// Structure representing keyboard event information.
///
typedef struct
{
    ///
    /// Size of this structure.
    ///
    size_t size;

    ///
    /// The type of keyboard event.
    ///
    KeyEventType type;

    ///
    /// Bit flags describing any pressed modifier keys. See
    /// cef_event_flags_t for values.
    ///
    uint32_t modifiers;

    ///
    /// The Windows key code for the key event. This value is used by the DOM
    /// specification. Sometimes it comes directly from the event (i.e. on
    /// Windows) and sometimes it's determined using a mapping function. See
    /// WebCore/platform/chromium/KeyboardCodes.h for the list of values.
    ///
    int windows_key_code;

    ///
    /// The actual key code genenerated by the platform.
    ///
    int native_key_code;

    ///
    /// Indicates whether the event is considered a "system key" event (see
    /// http://msdn.microsoft.com/en-us/library/ms646286(VS.85).aspx for details).
    /// This value will always be false on non-Windows platforms.
    ///
    int is_system_key;

    ///
    /// The character generated by the keystroke.
    ///
    uint_least16_t character;

    ///
    /// Same as |character| but unmodified by any concurrently-held modifiers
    /// (except shift). This is useful for working out shortcut keys.
    ///
    uint_least16_t unmodified_character;

    ///
    /// True if the focus is currently on an editable field on the page. This is
    /// useful for determining if standard key events should be intercepted.
    ///
    int focus_on_editable_field;
} KeyEvent;

typedef struct
{
    void (*on_state_change)(WebViewState state, void *context);
    void (*on_ime_rect)(Rect *rect, void *context);
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
    EXPORT void webview_mouse_click(void *webview_ptr, MouseEvent event, MouseButtonType button, bool pressed);

    //
    // Send a mouse wheel event to the browser. The |x| and |y| coordinates are
    // relative to the upper-left corner of the view. The |deltaX| and |deltaY|
    // values represent the movement delta in the X and Y directions
    // respectively. In order to scroll inside select popups with window
    // rendering disabled CefRenderHandler::GetScreenPoint should be implemented
    // properly.
    //
    EXPORT void webview_mouse_wheel(void *webview_ptr, MouseEvent event, int x, int y);

    //
    // Send a mouse move event to the browser. The |x| and |y| coordinates are
    // relative to the upper-left corner of the view.
    //
    EXPORT void webview_mouse_move(void *webview_ptr, MouseEvent event);

    //
    // Send a key event to the browser.
    //
    EXPORT void webview_keyboard(void *webview_ptr, KeyEvent event);

    //
    // Send a touch event to the browser.
    //
    EXPORT void webview_touch(void *webview_ptr, TouchEvent event);

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
