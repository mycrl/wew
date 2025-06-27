//
//  wew.cpp
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#ifdef MACOS
#include "include/wrapper/cef_library_loader.h"
#endif

#include "runtime.h"
#include "subprocess.h"
#include "webview.h"
#include "wew.h"

int get_result_code()
{
    return CefGetExitCode();
}

void run_message_loop()
{
    CefRunMessageLoop();
}

void quit_message_loop()
{
    CefQuitMessageLoop();
}

void poll_message_loop()
{
    CefDoMessageLoopWork();
}

int execute_subprocess(int argc, const char **argv)
{
#ifdef MACOS
    CefScopedLibraryLoader library_loader;
    if (!library_loader.LoadInHelper())
    {
        return -1;
    }
#endif

    auto main_args = get_main_args(argc, argv);
    return CefExecuteProcess(main_args, new ISubProcess, nullptr);
}

void *create_runtime(const RuntimeSettings *settings, RuntimeHandler handler)
{
#ifdef MACOS
    CefScopedLibraryLoader library_loader;
    if (!library_loader.LoadInMain())
    {
        return nullptr;
    }
#endif

    assert(settings != nullptr);

    CefSettings cef_settings;

    cef_log_severity_t a;
    cef_settings.no_sandbox = true;
    cef_settings.background_color = settings->background_color;
    cef_settings.external_message_pump = settings->external_message_pump;
    cef_settings.persist_session_cookies = settings->persist_session_cookies;
    cef_settings.disable_signal_handlers = settings->disable_signal_handlers;
    cef_settings.command_line_args_disabled = settings->command_line_args_disabled;
    cef_settings.windowless_rendering_enabled = settings->windowless_rendering_enabled;
    cef_settings.multi_threaded_message_loop = settings->multi_threaded_message_loop;
    cef_settings.log_severity = static_cast<cef_log_severity_t>(static_cast<int>(settings->log_severity));

    if (settings->cache_path != nullptr)
    {
        CefString(&cef_settings.cache_path).FromString(settings->cache_path);
    }

    if (settings->root_cache_path != nullptr)
    {
        CefString(&cef_settings.root_cache_path).FromString(settings->root_cache_path);
    }

    if (settings->browser_subprocess_path != nullptr)
    {
        CefString(&cef_settings.browser_subprocess_path).FromString(settings->browser_subprocess_path);
    }

    if (settings->framework_dir_path != nullptr)
    {
        CefString(&cef_settings.framework_dir_path).FromString(settings->framework_dir_path);
    }

    if (settings->main_bundle_path != nullptr)
    {
        CefString(&cef_settings.main_bundle_path).FromString(settings->main_bundle_path);
    }

    if (settings->javascript_flags != nullptr)
    {
        CefString(&cef_settings.javascript_flags).FromString(settings->javascript_flags);
    }

    if (settings->resources_dir_path != nullptr)
    {
        CefString(&cef_settings.resources_dir_path).FromString(settings->resources_dir_path);
    }

    if (settings->locales_dir_path != nullptr)
    {
        CefString(&cef_settings.locales_dir_path).FromString(settings->locales_dir_path);
    }

    if (settings->user_agent != nullptr)
    {
        CefString(&cef_settings.user_agent).FromString(settings->user_agent);
    }
    else
    {
        CefString(&cef_settings.locale).FromString("en-US");
    }

    if (settings->user_agent_product != nullptr)
    {
        CefString(&cef_settings.user_agent_product).FromString(settings->user_agent_product);
    }

    if (settings->locale != nullptr)
    {
        CefString(&cef_settings.locale).FromString(settings->locale);
    }

    if (settings->log_file != nullptr)
    {
        CefString(&cef_settings.log_file).FromString(settings->log_file);
    }

    return new Runtime{new IRuntime(settings, cef_settings, handler)};
}

bool execute_runtime(void *runtime, int argc, const char **argv)
{
    assert(runtime != nullptr);

    auto rt = static_cast<Runtime *>(runtime);
    auto main_args = get_main_args(argc, argv);
    return CefInitialize(main_args, rt->ref->GetCefSettings(), rt->ref, nullptr);
}

void close_runtime(void *runtime)
{
    assert(runtime != nullptr);

    auto rt = static_cast<Runtime *>(runtime);
    rt->ref->Close();
    delete rt;
}

void *create_webview(void *runtime, const char *url, const WebViewSettings *settings, WebViewHandler handler)
{
    assert(runtime != nullptr);
    assert(settings != nullptr);
    assert(url != nullptr);

    auto webview = static_cast<Runtime *>(runtime)->ref->CreateWebView(std::string(url), settings, handler);
    return new WebView{webview};
}

void close_webview(void *webview)
{
    assert(webview != nullptr);

    auto view = static_cast<WebView *>(webview);
    view->ref->Close();

    delete view;
}

void webview_mouse_click(void *webview, MouseEvent event, MouseButton button, bool pressed)
{
    assert(webview != nullptr);

    CefMouseEvent cef_event;
    cef_event.x = event.x;
    cef_event.y = event.y;
    cef_event.modifiers = event.modifiers;

    auto cef_button = static_cast<cef_mouse_button_type_t>(static_cast<int>(button));

    static_cast<WebView *>(webview)->ref->OnMouseClick(cef_event, cef_button, pressed);
}

void webview_mouse_wheel(void *webview, MouseEvent event, int x, int y)
{
    assert(webview != nullptr);

    CefMouseEvent cef_event;
    cef_event.x = event.x;
    cef_event.y = event.y;
    cef_event.modifiers = event.modifiers;

    static_cast<WebView *>(webview)->ref->OnMouseWheel(cef_event, x, y);
}

void webview_mouse_move(void *webview, MouseEvent event)
{
    assert(webview != nullptr);

    CefMouseEvent cef_event;
    cef_event.x = event.x;
    cef_event.y = event.y;
    cef_event.modifiers = event.modifiers;

    static_cast<WebView *>(webview)->ref->OnMouseMove(cef_event);
}

void webview_keyboard(void *webview, KeyEvent event)
{
    assert(webview != nullptr);

    CefKeyEvent cef_event;
    cef_event.modifiers = event.modifiers;
    cef_event.native_key_code = event.native_key_code;
    cef_event.windows_key_code = event.windows_key_code;
    cef_event.character = event.character;
    cef_event.unmodified_character = event.unmodified_character;
    cef_event.is_system_key = event.is_system_key;
    cef_event.focus_on_editable_field = event.focus_on_editable_field;
    cef_event.type = static_cast<cef_key_event_type_t>(static_cast<int>(event.type));

    static_cast<WebView *>(webview)->ref->OnKeyboard(cef_event);
}

void webview_touch(void *webview, TouchEvent event)
{
    assert(webview != nullptr);

    CefTouchEvent cef_event;
    cef_event.id = event.id;
    cef_event.x = event.x;
    cef_event.y = event.y;
    cef_event.radius_x = event.radius_x;
    cef_event.radius_y = event.radius_y;
    cef_event.pressure = event.pressure;
    cef_event.modifiers = event.modifiers;
    cef_event.rotation_angle = event.rotation_angle;
    cef_event.type = static_cast<cef_touch_event_type_t>(static_cast<int>(event.type));
    cef_event.pointer_type = static_cast<cef_pointer_type_t>(static_cast<int>(event.pointer_type));

    static_cast<WebView *>(webview)->ref->OnTouch(cef_event);
}

void webview_ime_composition(void *webview, const char *input)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->OnIMEComposition(std::string(input));
}

void webview_ime_set_composition(void *webview, const char *input, int x, int y)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->OnIMESetComposition(input, x, y);
}

void webview_send_message(void *webview, const char *message)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->SendMessage(std::string(message));
}

void webview_set_devtools_state(void *webview, bool is_open)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->SetDevToolsOpenState(is_open);
}

void webview_resize(void *webview, int width, int height)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->Resize(width, height);
}

const void *webview_get_window_handle(void *webview)
{
    assert(webview != nullptr);

    return static_cast<WebView *>(webview)->ref->GetWindowHandle();
}

void webview_set_focus(void *webview, bool enable)
{
    assert(webview != nullptr);

    static_cast<WebView *>(webview)->ref->SetFocus(enable);
}
