//
//  webview.cpp
//  webview
//
//  Created by Mr.Panda on 2023/4/26.
//

#include "webview.h"
#include "app.h"

typedef struct
{
    CefRefPtr<IBrowser> ref;
} Browser;

CefMainArgs get_main_args(int argc, char** argv)
{
#ifdef WIN32
    CefMainArgs main_args(::GetModuleHandleW(nullptr));
#elif LINUX
    CefMainArgs main_args(argc, argv);
#endif

    return main_args;
}

void execute_sub_process(int argc, char** argv)
{
    auto main_args = get_main_args(argc, argv);
    CefExecuteProcess(main_args, new IRenderApp, nullptr);
}

void* create_webview(WebviewOptions* settings, CreateWebviewCallback callback, void* ctx)
{
    assert(settings);
    assert(callback);

    App* app = new App;
    app->ref = new IApp(settings, callback, ctx);
    app->settings = settings;
    return app;
}

void* create_page(void* app_ptr,
                  PageOptions* settings,
                  PageObserver observer,
                  void* ctx)
{
    assert(app_ptr);
    assert(settings);

    auto app = (App*)app_ptr;

    Browser* browser = new Browser;
    browser->ref = app->ref->CreateBrowser(settings, observer, ctx);
    return browser;
}

int webview_run(void* app_ptr, int argc, char** argv)
{
    assert(app_ptr);

    auto app = (App*)app_ptr;

    auto main_args = get_main_args(argc, argv);
    CefExecuteProcess(main_args, app->ref, nullptr);

    CefSettings cef_settings;
    cef_settings.windowless_rendering_enabled = true;
    cef_settings.chrome_runtime = false;
    cef_settings.no_sandbox = true;
    cef_settings.background_color = 0x00ffffff;

    // macos not support the multi threaded message loop.
#ifdef MACOS
    cef_settings.multi_threaded_message_loop = false;
#else
    cef_settings.multi_threaded_message_loop = true;
#endif

    CefString(&cef_settings.locale).FromString("zh-CN");

    auto cache_path = app->settings->cache_path;
    if (cache_path != nullptr)
    {
        CefString(&cef_settings.cache_path).FromString(cache_path);
        CefString(&cef_settings.log_file).FromString(std::string(cache_path) + "/webview.log");
    }

    auto browser_subprocess_path = app->settings->browser_subprocess_path;
    if (browser_subprocess_path != nullptr)
    {
        CefString(&cef_settings.browser_subprocess_path).FromString(browser_subprocess_path);
    }

    assert(&cef_settings);
    if (!CefInitialize(main_args, cef_settings, app->ref, nullptr))
    {
        return -1;
    }

#ifdef MACOS
    CefRunMessageLoop();
#endif
    return 0;
}

void webview_exit(void* app_ptr)
{
    auto app = (App*)app_ptr;

    assert(app);

#ifdef MACOS
    CefQuitMessageLoop();
#endif
    CefShutdown();
    delete app;
}

void page_exit(void* browser)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->IClose();
    delete page;
}

void page_send_mouse_click(void* browser, MouseButtons button, bool pressed)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->OnMouseClick(button, pressed);
}

void page_send_mouse_click_with_pos(void* browser,
                                    MouseButtons button,
                                    bool pressed,
                                    int x,
                                    int y)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->OnMouseClickWithPosition(button, x, y, pressed);
}

void page_send_mouse_wheel(void* browser, int x, int y)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->OnMouseWheel(x, y);
}

void page_send_mouse_move(void* browser, int x, int y)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->OnMouseMove(x, y);
}

void page_send_keyboard(void* browser, int scan_code, bool pressed, Modifiers modifiers)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->OnKeyboard(scan_code, pressed, modifiers);
}

void page_send_touch(void* browser,
                     int id,
                     int x,
                     int y,
                     TouchEventType type,
                     TouchPointerType pointer_type)
{
    assert(browser);

    auto page = (Browser*)browser;

    // TouchEventType have the same value with cef_touch_event_type_t.
    // Same as TouchPointerType.
    page->ref->OnTouch(id, x, y, (cef_touch_event_type_t)type, (cef_pointer_type_t)pointer_type);
}

void page_bridge_call(void* browser, char* req, BridgeCallCallback callback, void* ctx)
{
    assert(browser);
    assert(req);
    assert(callback);

    auto page = (Browser*)browser;

    page->ref->BridgeCall(req, callback, ctx);
}

void page_set_devtools_state(void* browser, bool is_open)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->SetDevToolsOpenState(is_open);
}

void page_resize(void* browser, int width, int height)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->Resize(width, height);
}

const void* page_get_hwnd(void* browser)
{
    assert(browser);

    auto page = (Browser*)browser;

    auto hwnd = page->ref->GetHWND();
    return (void*)hwnd;
}

void page_send_ime_composition(void* browser, char* input)
{
    assert(browser);
    assert(input);

    auto page = (Browser*)browser;

    page->ref->OnIMEComposition(std::string(input));
}

void page_send_ime_set_composition(void* browser, char* input, int x, int y)
{
    assert(browser);
    assert(input);

    auto page = (Browser*)browser;

    page->ref->OnIMESetComposition(std::string(input), x, y);
}
